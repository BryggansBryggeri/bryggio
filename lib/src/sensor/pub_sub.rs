use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsConfig, ClientId,
    PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::sensor::Sensor;
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::convert::TryFrom;
use std::thread::sleep;
use std::time::Duration;

pub struct SensorClient<S>
where
    S: Sensor,
{
    id: ClientId,
    sensor: S,
    /// TODO: Make generic over PubSubClient
    client: NatsClient,
}

impl<S> SensorClient<S>
where
    S: Sensor,
{
    pub fn new(id: ClientId, sensor: S, config: &NatsConfig) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        SensorClient { id, sensor, client }
    }

    fn gen_meas_subject(&self) -> Subject {
        Subject(format!("sensor.{}.measurement", self.id))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorMsg {
    id: ClientId,
    pub(crate) meas: f32,
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

impl<S> PubSubClient for SensorClient<S>
where
    S: Sensor,
{
    fn client_loop(self) -> Result<(), PubSubError> {
        let supervisor = self.subscribe(&Subject(format!("command.sensor.{}", self.id)))?;
        let meas_sub = self.gen_meas_subject();
        loop {
            for _msg in supervisor.try_iter() {
                // Deal with supervisor command
            }
            let meas = self.sensor.get_measurement()?;
            self.publish(
                &meas_sub,
                &SensorMsg {
                    id: self.id.clone(),
                    meas,
                }
                .into(),
            )?;
            sleep(Duration::from_millis(500));
        }
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}
