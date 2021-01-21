use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;


use std::collections::{VecDeque, HashMap};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::thread::JoinHandle;

#[derive(Hash, Clone, Copy)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Shutdown,
}

impl LogLevel {
    fn name(&self) -> String {
        match self {
            LogLevel::Debug => String::from("DEBUG"),
            LogLevel::Info => String::from("INFO"),
            LogLevel::Warn => String::from("WARN"),
            LogLevel::Error => String::from("ERROR"),
            LogLevel::Shutdown => String::from("SHUTDOWN"),
        }
    }
}

trait Formatter: Clone + Send + 'static {
    fn to_line(&self, msg: &Message) -> String;
}

#[derive(Clone)]
struct JsonFormatter {}

impl Formatter for JsonFormatter {
    fn to_line(&self, msg: &Message) -> String {
        let line = serde_json::json!({
            "level": msg.level.name(),
            "msg": msg.message,
            "msg_id": msg.message_id(),
        });
        line.to_string()
    }
}

struct Config<T: Formatter> {
    last_msg_size: usize,
    formatter: T,
}

impl<T: Formatter> Clone for Config<T> {
    fn clone(&self) -> Self {
        Self {
            last_msg_size: self.last_msg_size,
            formatter: self.formatter.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.last_msg_size = source.last_msg_size;
        self.formatter = source.formatter.clone();
    }
}

#[derive(Hash)]
struct Message {
    level: LogLevel,
    message: String,
}

impl Message {
    pub fn message_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            level: self.level.into(),
            message: self.message.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.level = source.level;
        self.message = source.message.clone();
    }
}

struct Logger {
    log_sender: Sender<Message>,
    log_receiver_state: JoinHandle<()>,
}

impl Logger {
    fn new<T: Formatter>(c: Config<T>) -> Self {
//        let cfg: Config<T> = c.clone();
        let cfg_msg_size = c.last_msg_size;
        let msg_formatter: T = c.formatter.into();
        let (s, r) = channel();
        let j = thread::spawn(move || {
//            let cfg: Config<T> = cfg.into();
            let rc: Receiver<Message> = r.into();
            let mut last_msg_hash = VecDeque::new();
            let mut dup_msg: HashMap<u64, Message> = HashMap::new();
            let mut dup_msg_count: HashMap<u64, u64> = HashMap::new();

            loop {
                match rc.recv() {
                    Ok(msg) => {
                        match msg.level {
                            LogLevel::Shutdown => { break; }
                            _ => {}
                        }
                        let msg_id = msg.message_id();
                        if last_msg_hash.contains(&msg_id) {
                            *dup_msg_count.entry(msg_id).or_insert(1) += 1;
                            dup_msg.insert(msg_id, msg.clone());

//                            println!("Found duplicated message - {}", msg_formatter.to_line(&msg));
                        } else {
                            println!("Log {}", msg_formatter.to_line(&msg));
                            last_msg_hash.push_front(msg_id);
                            if cfg_msg_size <= last_msg_hash.len() {
                                match last_msg_hash.back() {
                                    Some(out_msg_id) => {
                                        match dup_msg.get(out_msg_id) {
                                            Some(out_msg) => {
                                                println!("Flush dup: {}, dup count {}", msg_formatter.to_line(&out_msg), dup_msg_count.get(&out_msg_id).unwrap_or(&0));
                                                dup_msg_count.remove(&out_msg_id);
                                                dup_msg.remove(&out_msg_id);
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                };
                                last_msg_hash.truncate(cfg_msg_size);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Err - {:?}", e);
                    }
                }
            }

            for (msg_id, msg) in dup_msg {
                println!("Dup flush on exit: {}, dup count {}", msg_formatter.to_line(&msg), dup_msg_count.get(&msg_id).unwrap_or(&0));
            }
        });

        Self {
            log_sender: s,
            log_receiver_state: j,
        }
    }

    fn log(&self, lv: LogLevel, msg: &str) {
        self.log_sender.send(Message {
            level: lv,
            message: msg.to_string(),
        }).expect("cannot send a log message");
    }

    fn shutdown(self) -> std::thread::Result<()> {
        self.log_sender.send(Message {
            level: LogLevel::Shutdown,
            message: String::from(""),
        }).map_err(|err| {
            println!("Error: {}", err)
        }).ok();
        self.log_receiver_state.join()
    }
}

fn main() {
    let cfg = Config {
        last_msg_size: 2,
        formatter: JsonFormatter {},
    };
    let l = Logger::new(cfg);

    l.log(LogLevel::Debug, "Hello");
    for _ in 1..10 {
        l.log(LogLevel::Info, "World");
    }
    for i in 1..10 {
        l.log(LogLevel::Info, format!("Message{}", i).as_str());
    }
    l.log(LogLevel::Error, "Rust");
    for _ in 1..=10 {
        l.log(LogLevel::Warn, "World");
    }

    l.shutdown().expect("Logger shutdown abnormally");
}
