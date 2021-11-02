use crate::logger::{debug, info};
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsClientConfig,
    ClientId, MessageParseError, PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::sensor::{Sensor, SensorError};
use crate::time::TimeStamp;
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub struct SensorClient {
    id: ClientId,
    sensor: Box<dyn Sensor>,
    client: NatsClient,
}

impl SensorClient {
    pub fn new(id: ClientId, sensor: Box<dyn Sensor>, config: &NatsClientConfig) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        SensorClient { id, sensor, client }
    }

    fn meas_subject(&self) -> Subject {
        Subject(format!("sensor.{}.measurement", self.id))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorMsg {
    pub(crate) id: ClientId,
    pub(crate) timestamp: TimeStamp,
    pub(crate) meas: Result<f32, SensorError>,
}

impl SensorMsg {
    pub fn subject(id: &ClientId) -> Subject {
        Subject(format!("sensor.{}.measurement", id))
    }
}

impl From<SensorMsg> for PubSubMsg {
    fn from(msg: SensorMsg) -> PubSubMsg {
        PubSubMsg(serde_json::to_string(&msg).expect("Can always serialize"))
    }
}

impl TryFrom<Message> for SensorMsg {
    type Error = MessageParseError;
    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        decode_nats_data(&msg.data)
    }
}

impl PubSubClient for SensorClient {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        info(
            &self,
            format!("Starting sensor with id '{}'", self.id),
            &format!("sensor.{}", self.id),
        );
        let supervisor = self.subscribe(&Subject(format!("command.sensor.{}", self.id)))?;
        let meas_sub = self.meas_subject();
        loop {
            for _msg in supervisor.try_iter() {
                // Deal with supervisor command
            }
            let meas = self.sensor.get_measurement();
            let timestamp = TimeStamp::now();
            debug(
                &self,
                format!("Sensor {}: {:?}", self.id, &meas),
                &format!("sensor.{}", self.id),
            );
            let msg = SensorMsg {
                id: self.id.clone(),
                timestamp,
                meas,
            };
            self.publish(&meas_sub, &msg.into())?;
        }
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}
