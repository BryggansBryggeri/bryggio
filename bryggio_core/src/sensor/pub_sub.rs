//! Publish-subscribe for sensors
//!
//! This module connects all sensors to the pub-sub network.
//! It provides a generic type [`SensorClient`] which wraps any type that implements the [`Sensor`]
//! trait and provides methods for publishing measurments to the pub-sub server and subscribing to
//! commands from other clients in the network.

use crate::logger::info;
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsClientConfig,
    ClientId, MessageParseError, PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::sensor::{Sensor, SensorError};
use crate::time::TimeStamp;
use async_nats::{Message, Subscriber};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// Generic pub-sub sensor client
pub struct SensorClient {
    /// Unique pub-sub client ID.
    id: ClientId,
    /// Any type that implements the [`Sensor`] trait.
    sensor: Box<dyn Sensor>,
    client: NatsClient,
}

impl SensorClient {
    pub async fn new(id: ClientId, sensor: Box<dyn Sensor>, config: &NatsClientConfig) -> Self {
        let client = NatsClient::try_new(config).await.unwrap();
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
        decode_nats_data(&msg.payload)
    }
}

use tokio::time::{sleep, Duration};
impl PubSubClient for SensorClient {
    async fn client_loop(mut self) -> Result<(), PubSubError> {
        info(
            &self,
            format!("Starting sensor with id '{}'", self.id),
            &format!("sensor.{}", self.id),
        )
        .await;
        let meas_sub = self.meas_subject();
        loop {
            let meas = self.sensor.get_measurement();
            sleep(Duration::from_millis(1000)).await;
            let timestamp = TimeStamp::now();
            let msg = SensorMsg {
                id: self.id.clone(),
                timestamp,
                meas,
            };
            let _ = self.publish(&meas_sub, &msg.into()).await?;
        }
    }

    async fn subscribe(&self, subject: &Subject) -> Result<Subscriber, PubSubError> {
        self.client.subscribe(subject).await
    }

    async fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg).await
    }
}
