pub struct ProcessLog {
    level: LogLevel,
}

impl ProcessLog {
    pub fn debug(&self, msg: &str) {
        if self.level >= LogLevel::Debug {
            self.write(msg);
        }
    }

    pub fn info(&self, msg: &str) {
        if self.level >= LogLevel::Info {
            self.write(msg);
        }
    }

    pub fn warning(&self, msg: &str) {
        if self.level >= LogLevel::Warning {
            self.write(msg);
        }
    }

    pub fn error(&self, msg: &str) {
        self.write(msg);
    }

    fn write(&self, msg: &str) {
        // TODO: Generic writer
        println!("{}", msg);
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ord() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(!(LogLevel::Debug > LogLevel::Info));
        assert!((LogLevel::Error > LogLevel::Info));
    }
}
