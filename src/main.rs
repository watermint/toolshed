use std::{thread, time};


use config::Config;
use formatter::{JsonFormatter};
use logger::Logger;
use loglevel::LogLevel;
use std::thread::sleep;
use std::fmt::Display;
use crate::writer::{ConsoleWriter, Writer, TeeWriter};
use crate::formatter::ConsoleFormatter;

mod loglevel;
mod message;
mod logger;
mod formatter;
mod config;
mod writer;

fn out(msg: &str, ctx: &[(&str, Box<dyn Display>)]) {
    println!("msg: {}", msg)
}

fn main() {
    out("hello", &[("Test", Box::new(1))]);

    let wf = || -> Box<dyn Writer> {
        let cjw = ConsoleWriter {
            formatter: JsonFormatter {}
        };
        let ccw = ConsoleWriter{
            formatter: ConsoleFormatter{},
        };
        let tw = TeeWriter {
            writers: vec![Box::new(cjw), Box::new(ccw)],
        };
        Box::new(tw)
    };
    let cfg = Config {
        dup_msg_scope: 2,
        writer_factory: wf,
    };
    let (l, lj) = Logger::new(&cfg);

    l.log(LogLevel::Debug, "Hello");
    for _ in 1..10 {
        l.log(LogLevel::Info, "World");
        sleep(time::Duration::from_millis(100));
    }
    for i in 1..10 {
        l.log(LogLevel::Info, format!("Message{}", i).as_str());
        sleep(time::Duration::from_millis(100));
    }
    let lc = l.clone();
    let j = thread::spawn(move || {
        for _ in 1..10 {
            lc.log(LogLevel::Info, format!("Fixed message from the other thread").as_str())
        }
        for i in 1..10 {
            lc.log(LogLevel::Info, format!("Message from the other thread {}", i).as_str())
        }
    });
    l.log(LogLevel::Error, "Rust");
    for _ in 1..=10 {
        l.log(LogLevel::Warn, "World");
    }
    j.join().map_err(|err| { println!("unable to join {:?}", err) }).ok();
    l.shutdown();

    lj.join().ok();
}
