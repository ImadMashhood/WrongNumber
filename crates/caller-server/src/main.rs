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
        properties["fact_references"]["maxItems"] = serde_json::json!(3);
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
impl Provider {
    fn respond(&self, c: &Context, mut reserve: impl FnMut() -> bool) -> Option<Reply> {
        let started = Instant::now();
        for attempt in 0..2 {
            let remaining = Duration::from_secs(20).checked_sub(started.elapsed())?;
            if remaining < Duration::from_secs(1) || !reserve() {
                return None;
            }
            let reply = self.respond_once(c, attempt == 1, remaining)?;
            if validate(c, &reply) && !repeated_reply(c, &reply.spoken_text) {
                return Some(reply);
            }
            eprintln!(
                "Caller reply failed contract or repeated prior wording; {}.",
                if attempt == 0 {
                    "requesting one live rewrite"
                } else {
                    "disconnecting after failed rewrite"
                }
            );
        }
        None
    }
    fn respond_once(&self, c: &Context, rewrite: bool, remaining: Duration) -> Option<Reply> {
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
        if rewrite {
            messages.push(serde_json::json!({"role":"system","content":"Your previous attempt failed validation or duplicated earlier wording and was not delivered. Obey the response schema exactly. For Opening: fact_references must be [], call_complete false, and all three behavior fields must be null. For Conversation: behavior.quote must be an exact excerpt from the latest dispatcher message or null; IDs must come from the supplied facts. Write a genuinely different in-character response to the latest dispatcher message. Change the conversational move, not just the first two words. Acknowledge frustration, explain a misunderstanding, ask one useful question, or establish a personal boundary as appropriate. Preserve all incident facts. Return the required JSON."}));
        }
        let mut body = serde_json::json!({"model":self.model,"messages":messages,"max_tokens":256,"temperature":0.85,"response_format":schema(c),"provider":{"require_parameters":true,"max_price":{"prompt":self.prices[0],"completion":self.prices[1]}},"stream":false});
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
                assert_eq!(body["max_tokens"], 256);
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
