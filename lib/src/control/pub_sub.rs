use crate::control::Control;
use crate::pub_sub::{
    nats_client::NatsClient, nats_client::NatsConfig, ClientId, PubSubClient, PubSubError,
    PubSubMsg, Subject,
};
use crate::sensor::SensorMsg;
use nats::{Message, Subscription};
use std::convert::TryFrom;

pub enum ControllerSubMsg {
    SetTarget(f32),
    Stop,
}

impl TryFrom<Message> for ControllerSubMsg {
    type Error = PubSubError;
    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let signal = String::from_utf8(value.data)
            .map_err(|err| PubSubError::MessageParse(err.to_string()))?
            .parse::<f32>()
            .map_err(|err| PubSubError::MessageParse(err.to_string()))?;
        Ok(ControllerSubMsg::SetTarget(signal))
    }
}

pub enum ControllerPubMsg {
    SetSignal(f32),
}

impl ControllerPubMsg {
    fn into_msg(&self) -> PubSubMsg {
        match self {
            ControllerPubMsg::SetSignal(signal) => PubSubMsg(format!("{}", signal)),
        }
    }
}

pub struct ControllerClient<C>
where
    C: Control,
{
    id: ClientId,
    actor_id: ClientId,
    sensor_id: ClientId,
    controller: C,
    client: NatsClient,
}

impl<C> ControllerClient<C>
where
    C: Control,
{
    pub fn new(
        id: ClientId,
        actor_id: ClientId,
        sensor_id: ClientId,
        controller: C,
        config: &NatsConfig,
    ) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        ControllerClient {
            id,
            actor_id,
            sensor_id,
            controller,
            client,
        }
    }

    fn gen_meas_subject(&self) -> Subject {
        Subject(format!("actor.{}.set_signal", self.actor_id))
    }
}

impl<C> PubSubClient for ControllerClient<C>
where
    C: Control,
{
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let supervisor = self.subscribe(&Subject(format!("controller.{}.*", self.id)))?;
        let sensor_subject = Subject(format!("sensor.{}.measurement", self.sensor_id));
        let sensor = self.subscribe(&sensor_subject)?;
        loop {
            if let Some(meas_msg) = sensor.next() {
                if let Ok(msg) = SensorMsg::try_from(meas_msg) {
                    self.controller.calculate_signal(Some(msg.meas));
                }
                self.publish(
                    &self.gen_meas_subject(),
                    &ControllerPubMsg::SetSignal(self.controller.get_control_signal()).into_msg(),
                )?;
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
