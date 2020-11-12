use crate::pub_sub::{
    nats_client::NatsClient, nats_client::NatsConfig, PubSubClient, PubSubError, PubSubMsg, Subject,
};
use nats::Subscription;
use serde::{Deserialize, Serialize};
use std::thread::sleep;
use std::time::Duration;
pub struct Log {
    level: LogLevel,
    client: NatsClient,
}

impl Log {
    pub fn new(config: &NatsConfig, level: LogLevel) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        Log { level, client }
    }

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

impl PubSubClient for Log {
    fn client_loop(self) -> Result<(), PubSubError> {
        let sensor = Subject(format!("sensor.*.measurement"));
        let sensor_sub = self.subscribe(&sensor)?;

        let control_sub = self.subscribe(&Subject(format!("actor.*.set_signal")))?;
        let actor_sub = self.subscribe(&Subject(format!("actor.*.current_signal")))?;
        let ui_sub = self.subscribe(&Subject(format!("ext_comm.ui.test")))?;

        loop {
            //if let Some(msg) = control_sub.try_next() {
            //    println!("LOG: Control {}", msg);
            //}
            //if let Some(msg) = sensor_sub.try_next() {
            //    println!("LOG: Sensor {}", msg);
            //}
            //if let Some(msg) = actor_sub.try_next() {
            //    println!("LOG: Actor {}", msg);
            //}
            if let Some(msg) = ui_sub.try_next() {
                println!("EXT_COMM: UI {}", msg);
            }
            sleep(Duration::from_millis(10));
        }
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
