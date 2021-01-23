use crate::writer::Writer;

pub struct Config {
    pub dup_msg_scope: usize,
    pub writer_factory: fn() -> Box<dyn Writer>,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            dup_msg_scope: self.dup_msg_scope,
            writer_factory: self.writer_factory,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.dup_msg_scope = source.dup_msg_scope;
        self.writer_factory = source.writer_factory;
    }
}
