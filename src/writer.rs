use crate::message::{Message, DuplicatedMessages};
use crate::formatter::Formatter;

pub trait Writer {
    fn write(&mut self, msg: &Message);

    fn write_dup(&mut self, msg: &DuplicatedMessages);
}

pub struct TeeWriter {
    pub(crate) writers: Vec<Box<dyn Writer>>,
}

impl Writer for TeeWriter {
    fn write(&mut self, msg: &Message) {
        for w in self.writers.iter_mut() {
            w.write(msg)
        }
    }

    fn write_dup(&mut self, msg: &DuplicatedMessages) {
        for w in self.writers.iter_mut() {
            w.write_dup(msg)
        }
    }
}

pub struct ConsoleWriter<T: Formatter> {
    pub formatter: T,
}

impl<T: Formatter> Writer for ConsoleWriter<T> {
    fn write(&mut self, msg: &Message) {
        println!("{}", self.formatter.to_line(msg))
    }

    fn write_dup(&mut self, msg: &DuplicatedMessages) {
        println!("{}", self.formatter.to_dup_line(msg))
    }
}

