//! Trusted optional provider process. The game executable never reads provider keys.
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use wrong_number_caller::*;
struct Provider {
    endpoint: String,
    model: String,
    key: String,
    client: reqwest::blocking::Client,
    prices: [f64; 2],
}
fn schema(c: &Context) -> serde_json::Value {
    let kind = c.kind;
    let allow_hangup = c.allow_hangup && kind == TurnKind::Conversation;
    let actions = match kind {
        TurnKind::Opening => vec!["Opening"],
        TurnKind::Conversation if allow_hangup => vec!["Answer", "Clarify", "Goodbye"],
        TurnKind::Conversation => vec!["Answer", "Clarify"],
    };
    let mut schema = serde_json::json!({"type":"json_schema","json_schema":{"name":"caller_reply","strict":true,"schema":{"type":"object","additionalProperties":false,"required":["spoken_text","emotion","proposed_action","fact_references","call_complete","behavior"],"properties":{"spoken_text":{"type":"string"},"emotion":{"type":"string","enum":["Calm","Anxious","Confused","Dry","Angry","Defensive","Embarrassed","Relieved","Panicked","Cooperative","Suspicious"]},"proposed_action":{"type":"string","enum":actions},"fact_references":{"type":"array","items":{"type":"string"}},"call_complete":{"type":"boolean","enum":if allow_hangup {vec![false,true]} else {vec![false]}},"behavior":{"type":"object","additionalProperties":false,"required":["event","quote","requested_disclosure"],"properties":{"event":{"type":["string","null"],"enum":[null,"Information","Clarification","Evidence","Contradiction","Accusation","Reassurance","Negotiation","Pressure","Threat","Trick","Irrelevant","Insult"]},"quote":{"type":["string","null"]},"requested_disclosure":{"type":["string","null"]}}}}}}});
    let properties = &mut schema["json_schema"]["schema"]["properties"];
    if kind == TurnKind::Opening {
        properties["fact_references"]["maxItems"] = serde_json::json!(0);
        for field in ["event", "quote", "requested_disclosure"] {
            properties["behavior"]["properties"][field] = serde_json::json!({"type":"null"});
        }
    } else {
        properties["fact_references"]["maxItems"] = serde_json::json!(8);
        if c.facts.is_empty() {
            properties["fact_references"]["maxItems"] = serde_json::json!(0);
        } else {
            properties["fact_references"]["items"]["enum"] =
                serde_json::json!(c.facts.iter().map(|f| &f.id).collect::<Vec<_>>());
        }
        let mut disclosure = vec![serde_json::Value::Null];
        disclosure.extend(c.facts.iter().map(|f| serde_json::json!(f.id)));
        properties["behavior"]["properties"]["requested_disclosure"]["enum"] =
            serde_json::json!(disclosure);
    }
    schema
}
// Narrow, deterministic checks request another real model response, never scripted speech.
fn quality_issue(c: &Context, r: &Reply) -> Option<&'static str> {
    let q = c.question.to_lowercase();
    let reply = r.spoken_text.to_lowercase();
    if [
        "i already told you",
        "what more do you need",
        "as i already said",
    ]
    .iter()
    .any(|v| reply.contains(v))
        && ["where", "when", "how long", "details", "did you"]
            .iter()
            .any(|v| q.contains(v))
    {
        return Some(
            "The dispatcher asked for details. Answer each new question using supplied facts, including location/timing when asked. State unknown details plainly. Do not complain that you already answered or repeat the initial report.",
        );
    }
    let social = [
        "are you single",
        "flirting",
        "hit on you",
        "don't care",
        "don't give a shit",
    ]
    .iter()
    .any(|v| q.contains(v));
    if social
        && reply
            .split(['.', '!', '?'])
            .map(str::trim)
            .filter(|s| s.split_whitespace().count() >= 7)
            .any(|sentence| {
                c.history.iter().any(|(_, old)| {
                    old.to_lowercase()
                        .split(['.', '!', '?'])
                        .any(|s| s.trim() == sentence)
                })
            })
    {
        return Some(
            "A whole sentence repeats your earlier incident summary. Answer the social remark in ONE short in-character sentence. Do not repeat the incident report or add a routine request for help.",
        );
    }
    let facts = format!(
        "{} {}",
        c.report,
        c.facts
            .iter()
            .map(|f| f.answer.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    )
    .to_lowercase();
    if [
        "someone is hurt",
        "someone's hurt",
        "they are injured",
        "they're injured",
        "bleeding",
    ]
    .iter()
    .any(|s| reply.contains(s))
        && !["injur", "hurt", "bleed"].iter().any(|s| facts.contains(s))
    {
        return Some(
            "No injury is established. Do not claim anyone is hurt, injured or bleeding. React to the dispatcher without adding incident facts.",
        );
    }
    if [
        "don't give a shit",
        "don't care",
        "fix that yourself",
        "fix it yourself",
    ]
    .iter()
    .any(|s| q.contains(s))
        && !["you caused", "you burned", "your fault"]
            .iter()
            .any(|s| q.contains(s))
        && ["i didn't", "i did not", "i haven't", "i just saw"]
            .iter()
            .any(|s| reply.contains(s))
    {
        return Some(
            "This is a refusal of help, NOT an accusation that you caused the incident. React to their indifference with frustration or disbelief in your own voice. Do not deny causing it or repeat the report.",
        );
    }
    if ["are you single", "flirting", "hit on you"]
        .iter()
        .any(|s| q.contains(s))
        && [
            "not sure",
            "don't know what",
            "what do you mean",
            "calling me a single",
        ]
        .iter()
        .any(|s| reply.contains(s))
    {
        return Some(
            "The dispatcher is flirting. You understand that social intent. Give a natural personal reaction; do not pretend the phrase is incomprehensible or a name.",
        );
    }
    if q.contains("not a pie")
        && ![
            "sorry",
            "misunderstood",
            "my mistake",
            "thought",
            "joke",
            "pun",
            "got it",
        ]
        .iter()
        .any(|s| reply.contains(s))
        && ["dessert", "pie-related", "not talking about pie"]
            .iter()
            .any(|s| reply.contains(s))
    {
        return Some(
            "The dispatcher is correcting your earlier literal pun. Acknowledge you misunderstood their meaning. Do not continue the dessert joke or deny your earlier wording.",
        );
    }
    None
}
impl Provider {
    fn respond(&self, c: &Context, mut reserve: impl FnMut() -> bool) -> Option<Reply> {
        let started = Instant::now();
        let mut feedback = None;
        for attempt in 0..2 {
            let remaining = Duration::from_secs(20).checked_sub(started.elapsed())?;
            if remaining < Duration::from_secs(1) || !reserve() {
                return None;
            }
            let reply = self.respond_once(c, feedback, remaining)?;
            if validate(c, &reply)
                && !repeated_reply(c, &reply.spoken_text)
                && quality_issue(c, &reply).is_none()
            {
                return Some(reply);
            }
            eprintln!(
                "Caller validation: contract={}, repeated={}, quality={:?}",
                validate(c, &reply),
                repeated_reply(c, &reply.spoken_text),
                quality_issue(c, &reply)
            );
            feedback = Some(
                quality_issue(c, &reply)
                    .unwrap_or("Follow the contract and avoid repeating earlier wording."),
            );
            eprintln!(
                "Caller reply failed contract or repeated prior wording; {}.",
                if attempt == 0 {
                    "requesting one live rewrite"
                } else {
                    "returning a retry error after failed rewrite"
                }
            );
        }
        None
    }
    fn respond_once(
        &self,
        c: &Context,
        feedback: Option<&str>,
        remaining: Duration,
    ) -> Option<Reply> {
        let system = include_str!("../../../assets/prompts/caller-system.txt");
        // Use real conversational roles instead of burying the exchange inside one JSON string.
        // The brief remains the first user message for strict fixture inspection.
        let mut brief = serde_json::to_value(c).ok()?;
        brief["history"] = serde_json::json!([]);
        brief["question"] = serde_json::json!("");
        brief.as_object_mut()?.remove("previous_reply_hashes");
        let mut messages = vec![
            serde_json::json!({"role":"system","content":system}),
            serde_json::json!({"role":"user","content":brief.to_string()}),
        ];
        for (question, answer) in &c.history {
            if !question.is_empty() {
                messages.push(serde_json::json!({"role":"user","content":question}));
            }
            messages.push(serde_json::json!({"role":"assistant","content":answer}));
        }
        if c.kind == TurnKind::Conversation {
            messages.push(serde_json::json!({"role":"user","content":c.question}));
        }
        if c.kind == TurnKind::Conversation {
            let q = c.question.to_lowercase();
            let hint = if ["are you single", "flirting", "hit on you"]
                .iter()
                .any(|s| q.contains(s))
            {
                Some(
                    "The latest dispatcher message is flirting with you, not asking about the incident. You understand this. React personally in character in ONE short sentence. Do not append the incident report or a request for help. Do not add an injury. Only supplied facts establish injuries; never infer them from someone being stuck.",
                )
            } else if [
                "don't give a shit",
                "don't care",
                "fix that yourself",
                "fix it yourself",
            ]
            .iter()
            .any(|s| q.contains(s))
                && !["you burned", "you caused", "your fault"]
                    .iter()
                    .any(|s| q.contains(s))
            {
                Some(
                    "The latest dispatcher message is refusing help / expressing indifference. It is NOT an accusation that you caused the incident. React naturally to being dismissed in ONE short sentence. Do not deny causing it or append the incident summary.",
                )
            } else if q.contains("not a pie") {
                Some(
                    "The dispatcher is correcting your earlier misunderstanding of their words. Acknowledge the correction. Stop making the dessert pun.",
                )
            } else {
                None
            };
            if let Some(hint) = hint {
                messages.push(serde_json::json!({"role":"system","content":hint}));
            }
        }
        if c.kind == TurnKind::Conversation {
            messages.push(serde_json::json!({"role":"system","content":"Before writing dialogue, identify the concrete questions in the latest user message. Answer every one from the provided facts; use a brief explicit unknown where information is absent. Do not treat requests for details as hostility. Use up to four short sentences if needed. Do not repeat the initial report in place of an answer. Fact references must include only the IDs supporting what you actually say (at most eight)."}));
        }
        if feedback.is_some() {
            messages.push(serde_json::json!({"role":"system","content":"Your previous attempt failed validation or duplicated earlier wording and was not delivered. Obey the response schema exactly. For Opening: fact_references must be [], call_complete false, and all three behavior fields must be null. For Conversation: behavior.quote must be an exact excerpt from the latest dispatcher message or null; IDs must come from the supplied facts. Write a genuinely different in-character response to the latest dispatcher message. Change the conversational move, not just the first two words. Acknowledge frustration, explain a misunderstanding, ask one useful question, or establish a personal boundary as appropriate. Preserve all incident facts. Return the required JSON."}));
        }
        if let Some(reason) = feedback {
            messages.push(serde_json::json!({"role":"system","content":format!("FINAL REWRITE REQUIREMENT: {reason} The latest dispatcher message is the one you must answer. Keep the reply brief and natural.")}));
        }
        let mut body = serde_json::json!({"model":self.model,"messages":messages,"max_tokens":384,"temperature":0.85,"response_format":schema(c),"provider":{"require_parameters":true,"max_price":{"prompt":self.prices[0],"completion":self.prices[1]}},"stream":false});
        if !self.endpoint.starts_with("https://openrouter.ai/") {
            body.as_object_mut()?.remove("provider");
        }
        if serde_json::to_vec(&body).ok()?.len() > 48_000 {
            return None;
        }
        let response = self
            .client
            .post(&self.endpoint)
            .timeout(remaining)
            .bearer_auth(&self.key)
            .json(&body)
            .send()
            .inspect_err(|_| eprintln!("Caller provider transport or timeout failure"))
            .ok()?;
        if !response.status().is_success() {
            eprintln!("Caller provider HTTP {}", response.status().as_u16());
            return None;
        }
        let mut bytes = Vec::new();
        response.take(16_385).read_to_end(&mut bytes).ok()?;
        if bytes.len() > 16_384 {
            return None;
        }
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .inspect_err(|_| eprintln!("Caller provider invalid outer JSON"))
            .ok()?;
        let r: Reply = serde_json::from_str(value["choices"][0]["message"]["content"].as_str()?)
            .inspect_err(|_| eprintln!("Caller provider invalid structured reply"))
            .ok()?;
        Some(r)
    }
}
fn price(name: &str) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|p| p.is_finite() && *p > 0.0 && *p <= 100.0)
        .unwrap_or(1.0)
}
fn main() {
    let address: SocketAddr = std::env::var("CALLER_BIND")
        .unwrap_or_else(|_| "127.0.0.1:47005".into())
        .parse()
        .expect("CALLER_BIND must be a loopback socket address");
    assert!(
        address.ip().is_loopback(),
        "Caller service binds only to loopback"
    );
    let key = std::env::var("CALLER_API_KEY")
        .or_else(|_| std::env::var("OPENROUTER_API_KEY"))
        .ok();
    let model = std::env::var("CALLER_MODEL").ok();
    let endpoint = std::env::var("CALLER_ENDPOINT")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1/chat/completions".into());
    let provider = if let (Some(key), Some(model)) = (key, model) {
        let url = reqwest::Url::parse(&endpoint).expect("Invalid caller endpoint");
        assert!(
            url.scheme() == "https"
                || (url.scheme() == "http"
                    && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"))),
            "Use HTTPS or a loopback test endpoint"
        );
        Some(Arc::new(Provider {
            endpoint,
            model,
            key,
            prices: [
                price("CALLER_MAX_INPUT_PRICE"),
                price("CALLER_MAX_OUTPUT_PRICE"),
            ],
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(20))
                .connect_timeout(Duration::from_secs(3))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
        }))
    } else {
        eprintln!(
            "Configure CALLER_API_KEY and CALLER_MODEL in this trusted process. No offline caller is available."
        );
        std::process::exit(1);
    };
    let max = std::env::var("CALLER_MAX_REQUESTS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100)
        .min(1000);
    let limiter = Arc::new(Mutex::new((Instant::now(), 0usize, 0usize, 0usize)));
    let listener = TcpListener::bind(address).expect("Caller service port unavailable");
    println!(
        "Caller service listening on {address}; {} mode; request budget {max}. No keys or dialogue are logged.",
        if provider.is_some() {
            "provider"
        } else {
            "unavailable"
        }
    );
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        {
            let mut limit = limiter.lock().unwrap();
            if limit.3 >= 6 {
                continue;
            }
            limit.3 += 1;
        }
        let provider = provider.clone();
        let limits = limiter.clone();
        std::thread::spawn(move || {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
            let read = read_frame(&mut stream, 32_000, Duration::from_secs(2));
            if let Ok(line) = read
                && let Ok(c) = serde_json::from_slice::<Context>(&line)
                && valid_context(&c)
            {
                let denied = std::cell::Cell::new(None);
                let reserve = || {
                    let mut l = limits.lock().unwrap();
                    if l.0.elapsed() >= Duration::from_secs(60) {
                        l.0 = Instant::now();
                        l.1 = 0;
                    }
                    let allowed = l.1 < 30 && l.2 < max;
                    if !allowed {
                        denied.set(Some(if l.2 >= max {
                            "request_budget"
                        } else {
                            "rate_limit"
                        }));
                    }
                    if allowed {
                        l.1 += 1;
                        l.2 += 1;
                    }
                    allowed
                };
                let reply = provider.as_ref().and_then(|p| p.respond(&c, reserve));
                if let Some(reply) = reply {
                    let response = ServiceReply {
                        reply,
                        source: "provider".into(),
                    };
                    let mut bytes = serde_json::to_vec(&response).unwrap();
                    bytes.push(b'\n');
                    let _ = stream.write_all(&bytes);
                } else {
                    let code = denied.get().unwrap_or(if provider.is_none() {
                        "unavailable"
                    } else {
                        "provider_failure"
                    });
                    eprintln!("Caller request failed: {code}");
                    let mut bytes = serde_json::to_vec(&serde_json::json!({"error":code})).unwrap();
                    bytes.push(b'\n');
                    let _ = stream.write_all(&bytes);
                }
                // Failure closes this request connection; there is no synthetic replacement.
            }
            limits.lock().unwrap().3 -= 1;
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;
    fn fixture(replies: Vec<bool>) -> (Provider, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let thread = std::thread::spawn(move || {
            for bad in replies {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = std::io::BufReader::new(&mut stream);
                let mut len = 0;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
                        len = v.trim().parse().unwrap();
                    }
                }
                let mut bytes = vec![0; len];
                reader.read_exact(&mut bytes).unwrap();
                let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(body["max_tokens"], 384);
                assert_eq!(body["response_format"]["type"], "json_schema");
                let mut c: Context =
                    serde_json::from_str(body["messages"][1]["content"].as_str().unwrap()).unwrap();
                c.question = body["messages"].as_array().unwrap().last().unwrap()["content"]
                    .as_str()
                    .unwrap()
                    .into();
                let mut reply = fixture_reply(&c);
                if bad {
                    reply.fact_references = vec!["invented_unavailable_fact".into()];
                }
                let answer=serde_json::json!({"choices":[{"message":{"content":serde_json::to_string(&reply).unwrap()}}]}).to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{answer}",
                    answer.len()
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        (
            Provider {
                endpoint: format!("http://{addr}"),
                model: "fixture-model".into(),
                key: "test-fixture-placeholder".into(),
                prices: [1.0, 1.0],
                client: reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(2))
                    .build()
                    .unwrap(),
            },
            thread,
        )
    }
    fn context() -> Context {
        Context {
            allow_hangup: false,
            previous_reply_hashes: vec![],
            version: VERSION,
            kind: TurnKind::Conversation,
            caller_name: "Alex Morgan".into(),
            location: "Pilot Road".into(),
            caller_private: String::new(),
            report: "Smoke".into(),
            personality: "anxious".into(),
            facts: vec![Knowledge {
                id: "smoke".into(),
                question: "What can you see?".into(),
                answer: "Black smoke from the dryer.".into(),
            }],
            history: vec![],
            question: "What can you see?".into(),
        }
    }
    #[test]
    fn compatible_http_adapter_accepts_only_a_valid_structured_response() {
        let (p, t) = fixture(vec![false]);
        assert!(p.respond(&context(), || true).is_some());
        t.join().unwrap();
        let (p, t) = fixture(vec![true, true]);
        assert!(p.respond(&context(), || true).is_none());
        t.join().unwrap();
    }
    #[test]
    fn opening_schema_cannot_request_evidence_or_classify_absent_dispatcher() {
        let mut c = context();
        c.kind = TurnKind::Opening;
        c.question.clear();
        c.allow_hangup = true;
        let schema = schema(&c);
        let p = &schema["json_schema"]["schema"]["properties"];
        assert_eq!(p["fact_references"]["maxItems"], 0);
        assert_eq!(p["call_complete"]["enum"], serde_json::json!([false]));
        for field in ["event", "quote", "requested_disclosure"] {
            assert_eq!(p["behavior"]["properties"][field]["type"], "null");
        }
    }
    #[test]
    fn invalid_opening_gets_one_live_repair_before_disconnect() {
        let mut c = context();
        c.kind = TurnKind::Opening;
        c.question.clear();
        let (p, t) = fixture(vec![true, false]);
        let mut attempts = 0;
        let reply = p
            .respond(&c, || {
                attempts += 1;
                true
            })
            .unwrap();
        assert_eq!(attempts, 2);
        assert!(validate(&c, &reply));
        assert_eq!(reply.proposed_action, Proposal::Opening);
        t.join().unwrap();
    }
    #[test]
    fn conversation_schema_only_exposes_available_fact_ids() {
        let c = context();
        let s = schema(&c);
        let p = &s["json_schema"]["schema"]["properties"];
        assert_eq!(
            p["fact_references"]["items"]["enum"],
            serde_json::json!(["smoke"])
        );
        assert_eq!(
            p["behavior"]["properties"]["requested_disclosure"]["enum"],
            serde_json::json!([null, "smoke"])
        );
    }
}

#[cfg(test)]
mod quality_tests {
    use super::*;
    #[test]
    fn refusal_and_flirting_have_explicit_rewrite_reasons() {
        let c:Context=serde_json::from_value(serde_json::json!({"version":4,"kind":"Conversation","caller_name":"Alex","location":"12 Main Street","report":"A person is stuck on a fire escape.","personality":"An anxious adult","caller_private":"No injuries are established.","facts":[],"history":[],"question":"I don't give a shit. Fix it yourself."})).unwrap();
        let mut r:Reply=serde_json::from_value(serde_json::json!({"spoken_text":"I didn't do anything to it.","emotion":"Defensive","proposed_action":"Answer","fact_references":[],"call_complete":false,"behavior":{"event":null,"quote":null,"requested_disclosure":null}})).unwrap();
        assert!(quality_issue(&c, &r).is_some());
        r.spoken_text = "You're refusing to help? Seriously?".into();
        assert!(quality_issue(&c, &r).is_none());
        r.spoken_text = "Someone is hurt.".into();
        assert!(quality_issue(&c, &r).is_some());
    }
}
