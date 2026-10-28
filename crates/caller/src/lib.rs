//! Provider-neutral dialogue contract. Facts stay authored; dialogue cannot execute game commands.
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
pub const VERSION: u32 = 1;
pub const CLARIFY: &str = "Could you ask about what happened or what I can see?";
pub const GOODBYE: &str = "Okay. Thank you for listening.";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Knowledge {
    pub id: String,
    pub question: String,
    pub answer: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub version: u32,
    pub report: String,
    pub personality: String,
    pub facts: Vec<Knowledge>,
    pub history: Vec<(String, String)>,
    pub question: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emotion {
    Calm,
    Anxious,
    Confused,
    Dry,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Proposal {
    Answer,
    Clarify,
    Goodbye,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub spoken_text: String,
    pub emotion: Emotion,
    pub proposed_action: Proposal,
    pub fact_references: Vec<String>,
    pub call_complete: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceReply {
    pub reply: Reply,
    pub source: String,
}
pub fn valid_context(c: &Context) -> bool {
    c.version == VERSION
        && !c.question.trim().is_empty()
        && c.question.len() <= 256
        && c.report.len() <= 512
        && c.personality.len() <= 80
        && c.facts.len() <= 8
        && c.facts
            .iter()
            .all(|f| f.id.len() <= 40 && f.question.len() <= 256 && f.answer.len() <= 512)
        && c.history.len() <= 8
        && c.history
            .iter()
            .all(|(a, b)| a.len() <= 256 && b.len() <= 512)
}
pub fn validate(c: &Context, r: &Reply) -> bool {
    if !valid_context(c) || r.fact_references.len() > 1 {
        return false;
    }
    match r.proposed_action {
        Proposal::Answer => {
            !r.call_complete
                && r.fact_references.len() == 1
                && c.facts
                    .iter()
                    .any(|f| r.fact_references[0] == f.id && r.spoken_text == f.answer)
        }
        Proposal::Clarify => {
            !r.call_complete && r.fact_references.is_empty() && r.spoken_text == CLARIFY
        }
        Proposal::Goodbye => {
            r.call_complete && r.fact_references.is_empty() && r.spoken_text == GOODBYE
        }
    }
}
pub fn scripted(c: &Context) -> Reply {
    let q = c.question.to_lowercase();
    let words: Vec<_> = q
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();
    let bye = words
        .iter()
        .any(|w| ["bye", "goodbye", "thanks"].contains(w));
    let fact = c
        .facts
        .iter()
        .max_by_key(|f| {
            let specific: &[&str] = match f.id.as_str() {
                "smoke" => &["smoke", "fire", "see", "dryer", "burning"],
                "story" => &["where", "moon", "happened", "game", "real", "location"],
                "symptoms" => &[
                    "urgent",
                    "symptoms",
                    "hurt",
                    "sick",
                    "appointment",
                    "breathing",
                ],
                "source" => &[
                    "noise",
                    "explosion",
                    "source",
                    "fire",
                    "kitchen",
                    "happened",
                ],
                "breathing" => &[
                    "awake",
                    "wake",
                    "breathing",
                    "breathe",
                    "hurt",
                    "conscious",
                    "okay",
                ],
                "window" => &[
                    "see", "window", "shop", "stole", "carrying", "happened", "weapon",
                ],
                _ => &[],
            };
            words
                .iter()
                .filter(|w| {
                    specific.contains(w) || (w.len() > 3 && f.question.to_lowercase().contains(**w))
                })
                .count()
        })
        .filter(|f| {
            // General questions are useful across caller templates; off-topic input clarifies.
            q.contains("what")
                || q.contains("tell")
                || q.contains("describe")
                || q.contains("detail")
                || f.question.to_lowercase().split_whitespace().any(|w| {
                    w.len() > 3 && q.contains(w.trim_matches(|c: char| !c.is_alphanumeric()))
                })
                || [
                    "smoke",
                    "fire",
                    "breath",
                    "awake",
                    "wake",
                    "urgent",
                    "symptom",
                    "where",
                    "window",
                    "noise",
                    "source",
                    "hurt",
                    "conscious",
                ]
                .iter()
                .any(|w| q.contains(w))
        });
    if bye {
        Reply {
            spoken_text: GOODBYE.into(),
            emotion: Emotion::Calm,
            proposed_action: Proposal::Goodbye,
            fact_references: vec![],
            call_complete: true,
        }
    } else if let Some(f) = fact {
        Reply {
            spoken_text: f.answer.clone(),
            emotion: Emotion::Anxious,
            proposed_action: Proposal::Answer,
            fact_references: vec![f.id.clone()],
            call_complete: false,
        }
    } else {
        Reply {
            spoken_text: CLARIFY.into(),
            emotion: Emotion::Confused,
            proposed_action: Proposal::Clarify,
            fact_references: vec![],
            call_complete: false,
        }
    }
}
/// One bounded JSON line per local service request. No credentials travel here.
pub fn service(address: std::net::SocketAddr, context: &Context) -> Result<ServiceReply, String> {
    let mut stream =
        std::net::TcpStream::connect_timeout(&address, std::time::Duration::from_millis(300))
            .map_err(|_| "Caller service unavailable")?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;
    let mut bytes = serde_json::to_vec(context).unwrap();
    bytes.push(b'\n');
    stream
        .write_all(&bytes)
        .map_err(|_| "Caller service write failed")?;
    let line = read_frame(&mut stream, 16_384, std::time::Duration::from_secs(5))?;
    let response: ServiceReply =
        serde_json::from_slice(&line).map_err(|_| "Invalid caller response")?;
    if !["provider", "scripted fallback", "scripted"].contains(&response.source.as_str())
        || !validate(context, &response.reply)
    {
        return Err("Caller response failed fact validation".into());
    }
    Ok(response)
}

/// Absolute deadline, even if a peer trickles bytes. Bounds apply before parsing.
pub fn read_frame(
    stream: &mut std::net::TcpStream,
    max: usize,
    timeout: std::time::Duration,
) -> Result<Vec<u8>, String> {
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(100)))
        .map_err(|_| "Cannot set caller deadline")?;
    let start = std::time::Instant::now();
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 1024];
    while start.elapsed() < timeout {
        match stream.read(&mut buffer) {
            Ok(0) => return Err("Caller service disconnected".into()),
            Ok(n) => {
                bytes.extend_from_slice(&buffer[..n]);
                if bytes.len() > max {
                    return Err("Caller frame too large".into());
                }
                if bytes.last() == Some(&b'\n') {
                    return Ok(bytes);
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(_) => return Err("Caller service read failed".into()),
        }
    }
    Err("Caller service timed out".into())
}
