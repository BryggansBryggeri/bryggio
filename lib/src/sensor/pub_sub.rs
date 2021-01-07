use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsConfig, ClientId,
    PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::sensor::{Sensor, SensorError};
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::convert::TryFrom;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Ord, PartialOrd, PartialEq, Eq)]
pub struct TimeStamp(pub(crate) u128);

impl TimeStamp {
    fn now() -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        TimeStamp(since_the_epoch.as_millis())
    }
}

pub struct SensorClient {
    id: ClientId,
    sensor: Box<dyn Sensor>,
    client: NatsClient,
}

impl SensorClient {
    pub fn new(id: ClientId, sensor: Box<dyn Sensor>, config: &NatsConfig) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        SensorClient { id, sensor, client }
    }

    fn meas_subject(&self) -> Subject {
        Subject(format!("sensor.{}.measurement", self.id))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorMsg {
    id: ClientId,
    timestamp: TimeStamp,
    pub(crate) meas: Option<f32>,
    err: Option<SensorError>,
}

impl Into<PubSubMsg> for SensorMsg {
    fn into(self) -> PubSubMsg {
        PubSubMsg(to_string(&self).expect("Can always serialize"))
    }
}

impl TryFrom<Message> for SensorMsg {
    type Error = PubSubError;
    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        let sensor_msg = decode_nats_data(&msg.data)?;
        Ok(sensor_msg)
    }
}

impl PubSubClient for SensorClient {
    fn client_loop(self) -> Result<(), PubSubError> {
        let supervisor = self.subscribe(&Subject(format!("command.sensor.{}", self.id)))?;
        let meas_sub = self.meas_subject();
        loop {
            for _msg in supervisor.try_iter() {
                // Deal with supervisor command
            }
            let (meas, err) = match self.sensor.get_measurement() {
                Ok(meas) => (Some(meas), None),
                Err(err) => (None, Some(err)),
            };
            let timestamp = TimeStamp::now();
            self.publish(
                &meas_sub,
                &SensorMsg {
                    id: self.id.clone(),
                    timestamp,
                    meas,
                    err,
                }
                .into(),
            )?;
            sleep(Duration::from_millis(2000));
        }
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}
