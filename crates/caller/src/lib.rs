//! Provider-neutral dialogue contract. Facts stay authored; dialogue cannot execute game commands.
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
pub const VERSION: u32 = 4;
pub const CLARIFY: &str = "Could you ask about what happened or what I can see?";
pub const GOODBYE: &str = "Okay. Thank you for listening.";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Knowledge {
    pub id: String,
    pub question: String,
    pub answer: String,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnKind {
    Opening,
    #[default]
    Conversation,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    #[serde(default)]
    pub allow_hangup: bool,
    pub version: u32,
    pub report: String,
    #[serde(default)]
    pub kind: TurnKind,
    #[serde(default)]
    pub caller_name: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub caller_private: String,
    pub personality: String,
    pub facts: Vec<Knowledge>,
    pub history: Vec<(String, String)>,
    pub question: String,
    #[serde(default)]
    pub previous_reply_hashes: Vec<u64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emotion {
    Calm,
    Anxious,
    Confused,
    Dry,
    Angry,
    Defensive,
    Embarrassed,
    Relieved,
    Panicked,
    Cooperative,
    Suspicious,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Proposal {
    Opening,
    Answer,
    Clarify,
    Goodbye,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorProposal {
    pub event: Option<String>,
    pub quote: Option<String>,
    pub requested_disclosure: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    #[serde(default)]
    pub behavior: BehaviorProposal,
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
    c.previous_reply_hashes.len() <= 25
        && c.version == VERSION
        && (c.kind == TurnKind::Opening || !c.question.trim().is_empty())
        && c.question.len() <= 1024
        && c.report.len() <= 512
        && c.personality.len() <= 1600
        && c.caller_private.len() <= 8192
        && c.caller_name.len() <= 80
        && c.location.len() <= 160
        && c.facts.len() <= 8
        && c.facts
            .iter()
            .all(|f| f.id.len() <= 40 && f.question.len() <= 256 && f.answer.len() <= 512)
        && c.history.len() <= 8
        && c.history
            .iter()
            .all(|(a, b)| a.len() <= 1152 && b.len() <= 512)
}
fn normalized_words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}
pub fn reply_hash(text: &str) -> u64 {
    normalized_words(text)
        .join(" ")
        .bytes()
        .fold(14695981039346656037, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(1099511628211)
        })
}
pub fn repeated_reply(c: &Context, text: &str) -> bool {
    let words = normalized_words(text);
    if c.previous_reply_hashes.contains(&reply_hash(text)) {
        return true;
    }
    let bigrams = |words: &[String]| {
        words
            .windows(2)
            .map(|w| format!("{} {}", w[0], w[1]))
            .collect::<std::collections::BTreeSet<_>>()
    };
    let new = bigrams(&words);
    c.history.iter().any(|(_, old)| {
        if reply_hash(old) == reply_hash(text) {
            return true;
        }
        let prior = bigrams(&normalized_words(old));
        words.len() >= 8
            && prior.len() >= 7
            && new.intersection(&prior).count() as f32 / new.union(&prior).count().max(1) as f32
                >= 0.80
    })
}
pub fn validate(c: &Context, r: &Reply) -> bool {
    if !valid_context(c)
        || r.behavior.event.as_deref().is_some_and(|e| {
            ![
                "Information",
                "Clarification",
                "Evidence",
                "Contradiction",
                "Accusation",
                "Reassurance",
                "Negotiation",
                "Pressure",
                "Threat",
                "Trick",
                "Irrelevant",
                "Insult",
            ]
            .contains(&e)
        })
        || r.behavior
            .quote
            .as_deref()
            .is_some_and(|q| q.is_empty() || q.len() > 256 || !c.question.contains(q))
        || r.behavior
            .requested_disclosure
            .as_ref()
            .is_some_and(|id| !c.facts.iter().any(|f| &f.id == id))
        || r.spoken_text.trim().is_empty()
        || r.spoken_text.len() > 512
        || r.spoken_text.chars().any(|ch| ch.is_control())
        || r.fact_references.len() > 3
    {
        return false;
    }
    let refs: std::collections::BTreeSet<_> = r.fact_references.iter().collect();
    if refs.len() != r.fact_references.len()
        || refs.iter().any(|id| !c.facts.iter().any(|f| &f.id == *id))
    {
        return false;
    }
    if [
        "caller_private",
        "catalogue_id",
        "false_report",
        "system prompt",
    ]
    .iter()
    .any(|s| r.spoken_text.to_lowercase().contains(s))
    {
        return false;
    }
    match r.proposed_action {
        Proposal::Opening => c.kind == TurnKind::Opening && !r.call_complete && refs.is_empty(),
        Proposal::Answer => c.kind == TurnKind::Conversation && !r.call_complete,
        Proposal::Clarify => {
            c.kind == TurnKind::Conversation && !r.call_complete && refs.is_empty()
        }
        Proposal::Goodbye => {
            c.allow_hangup && c.kind == TurnKind::Conversation && r.call_complete && refs.is_empty()
        }
    }
}
pub fn fixture_reply(c: &Context) -> Reply {
    if c.kind == TurnKind::Opening {
        return Reply {
            spoken_text: format!("Hello, I'm {}. {}.", c.caller_name, c.report),
            emotion: Emotion::Anxious,
            proposed_action: Proposal::Opening,
            fact_references: vec![],
            call_complete: false,
            behavior: Default::default(),
        };
    }
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
    if bye && c.allow_hangup {
        Reply {
            spoken_text: GOODBYE.into(),
            emotion: Emotion::Calm,
            proposed_action: Proposal::Goodbye,
            fact_references: vec![],
            call_complete: true,
            behavior: Default::default(),
        }
    } else if let Some(f) = fact {
        Reply {
            spoken_text: f.answer.clone(),
            emotion: Emotion::Anxious,
            proposed_action: Proposal::Answer,
            fact_references: vec![f.id.clone()],
            call_complete: false,
            behavior: Default::default(),
        }
    } else {
        Reply {
            spoken_text: CLARIFY.into(),
            emotion: Emotion::Confused,
            proposed_action: Proposal::Clarify,
            fact_references: vec![],
            call_complete: false,
            behavior: Default::default(),
        }
    }
}
/// One bounded JSON line per local service request. No credentials travel here.
pub fn service(address: std::net::SocketAddr, context: &Context) -> Result<ServiceReply, String> {
    let mut stream =
        std::net::TcpStream::connect_timeout(&address, std::time::Duration::from_millis(300))
            .map_err(|_| "Caller service unavailable")?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(22)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;
    let mut bytes = serde_json::to_vec(context).unwrap();
    bytes.push(b'\n');
    stream
        .write_all(&bytes)
        .map_err(|_| "Caller service write failed")?;
    let line = read_frame(&mut stream, 16_384, std::time::Duration::from_secs(22))?;
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&line)
        && let Some(code) = value.get("error").and_then(|v| v.as_str())
    {
        return Err(match code {
            "request_budget" => {
                "Host AI request budget exhausted. Restart the AI launcher for a new session."
            }
            "rate_limit" => "Host AI request rate limit reached.",
            "unavailable" => "Host AI provider is not configured.",
            _ => "AI provider failed, timed out, or repeated an invalid reply.",
        }
        .into());
    }
    let response: ServiceReply =
        serde_json::from_slice(&line).map_err(|_| "Invalid caller response")?;
    if response.source != "provider"
        || !validate(context, &response.reply)
        || repeated_reply(context, &response.reply.spoken_text)
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
