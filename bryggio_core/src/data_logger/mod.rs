//! Data logger
use serde::{Deserialize, de::DeserializeOwned, Serialize};
use crate::pub_sub::ClientId;
use crate::time::TimeStamp;
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsClientConfig,
    MessageParseError, PubSubClient, PubSubError, PubSubMsg, Subject,
};
use std::fmt::Debug as DebugTrait;
use nats::Subscription;
use crate::sensor::SensorMsg;


pub struct DataLogger {
    client_id: ClientId,
    client: NatsClient,
}


impl PubSubClient for DataLogger {
    fn client_loop(self) -> Result<(), PubSubError> {
        let sensor_sub = self.subscribe(&Subject(String::from("sensor.>")))?;
        loop {
            if let Some(msg) = sensor_sub.next() {
                let data = decode_nats_data::<SensorMsg>(&msg.data).expect("Failed decoding data");
                println!("{:?}", data);
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

