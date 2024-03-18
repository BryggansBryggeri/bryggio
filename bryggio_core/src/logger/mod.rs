use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsClientConfig,
    MessageParseError, PubSubClient, PubSubError, PubSubMsg, Subject,
};
use derive_more::{Display, From};
use nats::Subscription;
use serde::{Deserialize, Serialize};

pub fn debug<T: Into<LogMsg>, C: PubSubClient>(client: &C, msg: T, sub_subject: &str) {
    log(client, msg, sub_subject, LogLevel::Debug);
}

pub fn info<T: Into<LogMsg>, C: PubSubClient>(client: &C, msg: T, sub_subject: &str) {
    log(client, msg, sub_subject, LogLevel::Info);
}

pub fn _warning<T: Into<LogMsg>, C: PubSubClient>(client: &C, msg: T, sub_subject: &str) {
    log(client, msg, sub_subject, LogLevel::Warning);
}

pub fn error<T: Into<LogMsg>, C: PubSubClient>(client: &C, msg: T, sub_subject: &str) {
    log(client, msg, sub_subject, LogLevel::Error);
}

fn log<T: Into<LogMsg>, C: PubSubClient>(client: &C, msg: T, sub_subject: &str, level: LogLevel) {
    let msg: LogMsg = msg.into();
    let subj = Subject(format!("{}.{}", level.main_subject(), sub_subject));

    let msg = match serde_json::to_string(&msg) {
        Ok(msg) => PubSubMsg(msg),
        Err(err) => {
            println!("Log error: {}", err);
            return;
        }
    };
    match client.publish(&subj, &msg) {
        Ok(_) => {}
        Err(err) => println!("Log error: {}", err),
    };
}

pub struct Log {
    level: LogLevel,
    client: NatsClient,
}

impl PubSubClient for Log {
    fn client_loop(self) -> Result<(), PubSubError> {
        let log_sub = self.subscribe(&Subject(String::from("log.>")))?;
        loop {
            if let Some(msg) = log_sub.next() {
                match LogLevel::from_msg_subject(&msg.subject) {
                    Ok(log_level) => match decode_nats_data::<LogMsg>(&msg.data) {
                        Ok(msg) => self.log(&msg.0, log_level),
                        Err(err) => self.error(&err.to_string()),
                    },
                    Err(err) => self.error(&err.to_string()),
                };
            }
        }
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}

impl Log {
    pub fn new(config: &NatsClientConfig, level: LogLevel) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        Log { level, client }
    }

    pub fn log(&self, msg: &str, level: LogLevel) {
        match level {
            LogLevel::Debug => self.debug(msg),
            LogLevel::Info => self.info(msg),
            LogLevel::Warning => self.warning(msg),
            LogLevel::Error => self.error(msg),
        }
    }

    pub fn debug(&self, msg: &str) {
        if self.level <= LogLevel::Debug {
            self.write(msg, LogLevel::Debug);
        }
    }

    pub fn info(&self, msg: &str) {
        if self.level <= LogLevel::Info {
            self.write(msg, LogLevel::Info);
        }
    }

    pub fn warning(&self, msg: &str) {
        if self.level <= LogLevel::Warning {
            self.write(msg, LogLevel::Warning);
        }
    }

    pub fn error(&self, msg: &str) {
        self.write(msg, LogLevel::Error);
    }

    fn write(&self, msg: &str, level: LogLevel) {
        println!("{}: {}", level, msg);
    }
}

#[derive(Deserialize, Serialize, Display, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl TryFrom<&str> for LogLevel {
    type Error = MessageParseError;
    fn try_from(value: &str) -> Result<Self, MessageParseError> {
        match value {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warning" => Ok(LogLevel::Warning),
            "error" => Ok(LogLevel::Error),
            _ => Err(MessageParseError::InvalidSubject(Subject(String::from(
                value,
            )))),
        }
    }
}

impl LogLevel {
    pub fn is_debug(&self) -> bool {
        self <= &LogLevel::Debug
    }

    fn from_msg_subject(subject: &str) -> Result<Self, MessageParseError> {
        let mut tmp = subject.split('.');
        tmp.next();
        let log_level = tmp
            .next()
            .ok_or_else(|| MessageParseError::InvalidSubject(Subject::from(subject)))?;
        LogLevel::try_from(log_level)
    }

    fn main_subject(&self) -> Subject {
        let str_ = match self {
            LogLevel::Debug => "log.debug",
            LogLevel::Info => "log.info",
            LogLevel::Warning => "log.warning",
            LogLevel::Error => "log.error",
        };
        Subject(String::from(str_))
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, From)]
pub struct LogMsg(pub(crate) String);

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
