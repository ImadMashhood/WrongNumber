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
fn schema(kind: TurnKind) -> serde_json::Value {
    let actions = match kind {
        TurnKind::Opening => vec!["Opening"],
        TurnKind::Conversation => vec!["Answer", "Clarify", "Goodbye"],
    };
    serde_json::json!({"type":"json_schema","json_schema":{"name":"caller_reply","strict":true,"schema":{"type":"object","additionalProperties":false,"required":["spoken_text","emotion","proposed_action","fact_references","call_complete"],"properties":{"spoken_text":{"type":"string"},"emotion":{"type":"string","enum":["Calm","Anxious","Confused","Dry"]},"proposed_action":{"type":"string","enum":actions},"fact_references":{"type":"array","items":{"type":"string"}},"call_complete":{"type":"boolean"}}}}})
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
            if !repeated_reply(c, &reply.spoken_text) {
                return Some(reply);
            }
            eprintln!(
                "Caller repeated prior wording; {}.",
                if attempt == 0 {
                    "requesting one live rewrite"
                } else {
                    "disconnecting after repeated rewrite"
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
            messages.push(serde_json::json!({"role":"system","content":"Your previous attempt duplicated earlier wording and was not delivered. Write a genuinely different in-character response to the latest dispatcher message. Change the conversational move, not just the first two words. Acknowledge frustration, explain a misunderstanding, ask one useful question, or establish a personal boundary as appropriate. Preserve all incident facts. Return the required JSON."}));
        }
        let mut body = serde_json::json!({"model":self.model,"messages":messages,"max_tokens":256,"temperature":0.85,"response_format":schema(c.kind),"provider":{"require_parameters":true,"max_price":{"prompt":self.prices[0],"completion":self.prices[1]}},"stream":false});
        if !self.endpoint.starts_with("https://openrouter.ai/") {
            body.as_object_mut()?.remove("provider");
        }
        if serde_json::to_vec(&body).ok()?.len() > 32_000 {
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
        if !validate(c, &r) {
            eprintln!(
                "Caller contract rejected reply: phase {:?}, action {:?}, bytes {}, references {}",
                c.kind,
                r.proposed_action,
                r.spoken_text.len(),
                r.fact_references.len()
            );
            return None;
        }
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
            let read = read_frame(&mut stream, 20_000, Duration::from_secs(2));
            if let Ok(line) = read
                && let Ok(c) = serde_json::from_slice::<Context>(&line)
                && valid_context(&c)
            {
                let reserve = || {
                    let mut l = limits.lock().unwrap();
                    if l.0.elapsed() >= Duration::from_secs(60) {
                        l.0 = Instant::now();
                        l.1 = 0;
                    }
                    let allowed = l.1 < 30 && l.2 < max;
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
    fn fixture(bad: bool) -> (Provider, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let thread = std::thread::spawn(move || {
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
            let c: Context =
                serde_json::from_str(body["messages"][1]["content"].as_str().unwrap()).unwrap();
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
        let (p, t) = fixture(false);
        assert!(p.respond(&context(), || true).is_some());
        t.join().unwrap();
        let (p, t) = fixture(true);
        assert!(p.respond(&context(), || true).is_none());
        t.join().unwrap();
    }
}
