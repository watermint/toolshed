use crate::message::{Message, DuplicatedMessages};

pub trait Formatter {
    /// Format message to the message line.
    fn to_line(&self, msg: &Message) -> String;

    /// Format message to the message line for duplicated messages.
    fn to_dup_line(&self, dm: &DuplicatedMessages) -> String;
}

pub struct JsonFormatter {}

impl Formatter for JsonFormatter {
    fn to_line(&self, msg: &Message) -> String {
        let ts = msg.timestamp.to_rfc3339();
        let line = serde_json::json!({
            "ts": ts,
            "level": msg.level.name(),
            "msg": msg.message,
            "msg_id": msg.message_id(),
        });
        line.to_string()
    }

    fn to_dup_line(&self, dm: &DuplicatedMessages) -> String {
        let msg = dm.message();
        let (ts, te) = dm.time_range();
        let line = serde_json::json!({
            "ts": ts.to_rfc3339(),
            "te": te.to_rfc3339(),
            "level": msg.level.name(),
            "msg": msg.message,
            "msg_id": msg.message_id(),
            "msg_dup": dm.count(),
        });
        line.to_string()
    }
}

pub struct ConsoleFormatter{}

impl Formatter for ConsoleFormatter {
    fn to_line(&self, msg: &Message) -> String {
        format!("{} [{}] {}", msg.timestamp.format("%H:%M:%S"), msg.level.name(), msg.message)
    }

    fn to_dup_line(&self, dm: &DuplicatedMessages) -> String {
        format!("{} [{}] {} (Duplicated", dm.time_range().0.format("%H:%M:%S"), dm.message().level.name(), dm.message().message)
    }
}