use crate::control::{Control, State};
use crate::logger::error;
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsConfig, ClientId,
    PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::sensor::SensorMsg;
use crate::supervisor::pub_sub::SupervisorSubMsg;
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

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
}

impl PubSubClient for ControllerClient {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let kill_cmd = self.subscribe(&SupervisorSubMsg::subject(&self.id, "kill"))?;
        let controller = self.subscribe(&ControllerSubMsg::subject(&self.id))?;
        let sensor = self.subscribe(&SensorMsg::subject(&self.sensor_id))?;
        let mut state = State::Active;
        while state == State::Active {
            if let Some(msg) = kill_cmd.try_next() {
                msg.respond(format!("{}", self.controller.get_target()))
                    .map_err(|err| {
                        PubSubError::Client(format!("could not respond: '{}'.", err.to_string()))
                    })?;
                state = State::Inactive;
            }

            if let Some(msg) = controller.try_next() {
                // TODO: Match and log error
                match ControllerSubMsg::try_from(msg) {
                    Ok(msg) => match msg {
                        ControllerSubMsg::SetTarget(new_target) => {
                            self.controller.set_target(new_target)
                        }
                    },
                    Err(err) => log_error(&self, &format!("{}", err.to_string())),
                };
            }

            if let Some(meas_msg) = sensor.next() {
                if let Ok(msg) = SensorMsg::try_from(meas_msg) {
                    self.controller.calculate_signal(msg.meas);
                }
                self.publish(
                    &ControllerPubMsg::subject(&self.actor_id),
                    &ControllerPubMsg::SetSignal(self.controller.get_control_signal()).into(),
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

pub fn log_error(client: &ControllerClient, msg: &str) {
    error(
        client,
        String::from(msg),
        &format!("controller.{}", &client.id),
    );
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ControllerSubMsg {
    #[serde(rename = "set_target")]
    SetTarget(f32),
}

impl ControllerSubMsg {
    pub fn subject(id: &ClientId) -> Subject {
        Subject(format!("controller.{}.set_target", id))
    }
}

impl TryFrom<Message> for ControllerSubMsg {
    type Error = PubSubError;
    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        let new_target: f32 = decode_nats_data(&msg.data)?;
        Ok(ControllerSubMsg::SetTarget(new_target))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ControllerPubMsg {
    #[serde(rename = "set_signal")]
    SetSignal(f32),
    #[serde(rename = "new_target")]
    NewTarget(f32),
}

impl ControllerPubMsg {
    pub fn subject(id: &ClientId) -> Subject {
        Subject(format!("actor.{}.set_signal", id))
    }
}

impl Into<PubSubMsg> for ControllerPubMsg {
    fn into(self) -> PubSubMsg {
        match self {
            ControllerPubMsg::SetSignal(signal) => {
                PubSubMsg(serde_json::to_string(&signal).expect("Unexpected serialize error."))
            }
            ControllerPubMsg::NewTarget(target) => {
                PubSubMsg(serde_json::to_string(&target).expect("Unexpected serialize error."))
            }
        }
    }
}
