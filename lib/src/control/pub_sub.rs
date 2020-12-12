use crate::control::{Control, State};
use crate::pub_sub::{
    nats_client::NatsClient, nats_client::NatsConfig, ClientId, PubSubClient, PubSubError,
    PubSubMsg, Subject,
};
use crate::sensor::SensorMsg;
use crate::supervisor::SUPERVISOR_TOPIC;
use nats::{Message, Subscription};
use std::convert::TryFrom;

pub enum ControllerSubMsg {
    SetTarget(f32),
    _Stop,
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
    fn into_msg(self) -> PubSubMsg {
        match self {
            ControllerPubMsg::SetSignal(signal) => PubSubMsg(format!("{}", signal)),
        }
    }
}

pub struct ControllerClient {
    id: ClientId,
    actor_id: ClientId,
    sensor_id: ClientId,
    controller: Box<dyn Control>,
    client: NatsClient,
}

impl ControllerClient {
    pub fn new(
        id: ClientId,
        actor_id: ClientId,
        sensor_id: ClientId,
        controller: Box<dyn Control>,
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

impl PubSubClient for ControllerClient {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let supervisor =
            self.subscribe(&Subject(format!("{}.kill.{}", SUPERVISOR_TOPIC, self.id)))?;
        let sensor_subject = Subject(format!("sensor.{}.measurement", self.sensor_id));
        let sensor = self.subscribe(&sensor_subject)?;
        let mut state = State::Active;
        while state == State::Active {
            if let Some(_msg) = supervisor.try_next() {
                println!("Exiting control loop");
                state = State::Inactive;
            }
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
        Ok(())
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}
