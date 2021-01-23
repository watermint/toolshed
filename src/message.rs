use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chrono::{DateTime, Local};

use crate::loglevel::LogLevel;

pub struct Message {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub message: String,
}

impl Message {
    pub fn new(level: LogLevel, message: &str) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            message: message.to_string(),
        }
    }

    pub fn message_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for Message {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // do not compute hash for the timestamp
        self.level.hash(state);
        self.message.hash(state);
    }

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H) where
        Self: Sized, {
        for d in data {
            d.hash(state);
        }
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            timestamp: self.timestamp.clone(),
            level: self.level.into(),
            message: self.message.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.timestamp = source.timestamp.clone();
        self.level = source.level;
        self.message = source.message.clone();
    }
}

pub struct DuplicatedMessages {
    dup_count: u64,
    dup_msg: Message,
    dup_start: DateTime<Local>,
    dup_end: DateTime<Local>,
}

impl DuplicatedMessages {
    pub fn new(msg: &Message) -> Self {
        Self {
            dup_count: 1,
            dup_msg: msg.clone(),
            dup_start: msg.timestamp.clone(),
            dup_end: msg.timestamp.clone(),
        }
    }

    pub fn add(&mut self, msg: &Message) {
        self.dup_count += 1;
        self.dup_msg = msg.clone();
        if self.dup_end < msg.timestamp {
            self.dup_end = msg.timestamp.clone()
        }
        if msg.timestamp < self.dup_start {
            self.dup_start = msg.timestamp.clone()
        }
    }

    pub fn message(&self) -> Message {
        self.dup_msg.clone()
    }

    pub fn count(&self) -> u64 {
        self.dup_count
    }

    pub fn time_range(&self) -> (DateTime<Local>, DateTime<Local>) {
        (self.dup_start, self.dup_end)
    }
}