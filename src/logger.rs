use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use crate::config::Config;
use crate::loglevel::LogLevel;
use crate::message::{Message, DuplicatedMessages};


pub struct Logger {
    log_sender: Sender<Message>,
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Self {
            log_sender: self.log_sender.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.log_sender = source.log_sender.clone();
    }
}

fn receiver(r: Receiver<Message>, c: &Config) {
    let mut last_msg_hash = VecDeque::new();
    let mut dup_msg: HashMap<u64, DuplicatedMessages> = HashMap::new();
    let mut writer = (c.writer_factory)();

    loop {
        match r.recv() {
            Ok(msg) => {
                match msg.level {
                    LogLevel::Shutdown => { break; }
                    _ => {}
                }
                let msg_id = msg.message_id();
                if last_msg_hash.contains(&msg_id) {
                    let dm = dup_msg.entry(msg_id).or_insert(DuplicatedMessages::new(&msg));
                    dm.add(&msg);
                } else {
                    writer.write(&msg);
                    last_msg_hash.push_front(msg_id);
                    if c.dup_msg_scope <= last_msg_hash.len() {
                        match last_msg_hash.back() {
                            Some(out_msg_id) => {
                                match dup_msg.get(out_msg_id) {
                                    Some(out_msg) => {
                                        writer.write_dup(&out_msg);
                                        dup_msg.remove(&out_msg_id);
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        };
                        last_msg_hash.truncate(c.dup_msg_scope);
                    }
                }
            }
            Err(e) => {
                println!("Err - {:?}", e);
            }
        }
    }

    for (_, msg) in dup_msg {
        writer.write_dup(&msg)
    }
}

impl Logger {
    pub fn new(c: &Config) -> (Self, JoinHandle<()>) {
        let (s, r) = channel();
        let cf = c.clone();
        let j = thread::spawn(move || {
            receiver(r, &cf)
        });

        (Self { log_sender: s }, j)
    }

    pub fn log(&self, lv: LogLevel, msg: &str) {
        self.log_sender.send(Message::new(lv, msg)).expect("cannot send a log message")
    }

    pub fn shutdown(self) {
        self.log_sender.send(Message::new(LogLevel::Shutdown, "")).expect("unable to send shutdown message")
    }
}
