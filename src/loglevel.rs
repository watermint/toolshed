#[derive(Hash, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Shutdown,
}

impl LogLevel {
    pub fn name(&self) -> String {
        match self {
            LogLevel::Debug => String::from("DEBUG"),
            LogLevel::Info => String::from("INFO"),
            LogLevel::Warn => String::from("WARN"),
            LogLevel::Error => String::from("ERROR"),
            LogLevel::Shutdown => String::from("SHUTDOWN"),
        }
    }
}
