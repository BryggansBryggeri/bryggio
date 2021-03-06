use crate::actor::pub_sub::SignalMsg;
use crate::control::ControllerType;
use crate::control::{Control, State};
use crate::logger::{debug, error, info};
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsConfig, ClientId,
    PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::sensor::SensorMsg;
use crate::supervisor::pub_sub::SupervisorPubMsg;
use crate::time::TimeStamp;
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub struct ControllerClient {
    id: ClientId,
    actor_id: ClientId,
    sensor_id: ClientId,
    controller: Box<dyn Control>,
    client: NatsClient,
    type_: ControllerType,
}

impl ControllerClient {
    pub fn new(
        id: ClientId,
        actor_id: ClientId,
        sensor_id: ClientId,
        controller: Box<dyn Control>,
        config: &NatsConfig,
        type_: ControllerType,
    ) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        ControllerClient {
            id,
            actor_id,
            sensor_id,
            controller,
            client,
            type_,
        }
    }

    fn status_update(&self) {
        let status_update = ControllerPubMsg::Status {
            id: self.id.clone(),
            timestamp: TimeStamp::now(),
            target: self.controller.get_target(),
            type_: self.type_.clone(),
        };
        if let Err(err) = self.publish(&status_update.subject(&self.id), &status_update.into()) {
            log_error(
                &self,
                &format!("Could not publish status update: {}", err.to_string()),
            );
        };
    }
}

impl PubSubClient for ControllerClient {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let kill_cmd = self.subscribe(
            &SupervisorPubMsg::KillClient {
                client_id: self.id.clone(),
            }
            .subject(),
        )?;
        let controller = self.subscribe(&ControllerSubMsg::subject(&self.id))?;
        let sensor = self.subscribe(&SensorMsg::subject(&self.sensor_id))?;
        let mut state = State::Active;
        log_info(
            &self,
            &format!("starting contr. client: {}: {:?}", &self.id, &self.type_),
        );
        self.status_update();
        while state == State::Active {
            if let Some(msg) = kill_cmd.try_next() {
                // TODO: Proper Status PubMsg.
                log_info(&self, "killing contr. client");
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
                            log_debug(
                                &self,
                                &format!(
                                    "Setting target '{}' for controller '{}'",
                                    new_target, self.id
                                ),
                            );
                            self.controller.set_target(new_target)
                        }
                    },
                    Err(err) => log_error(&self, &err.to_string()),
                };
            }

            self.status_update();

            if let Some(meas_msg) = sensor.next() {
                if let Ok(msg) = SensorMsg::try_from(meas_msg) {
                    self.controller.calculate_signal(msg.meas.ok());
                }
                let msg = ControllerPubMsg::SetSignal(SignalMsg {
                    id: self.actor_id.clone(),
                    timestamp: TimeStamp::now(),
                    signal: self.controller.get_control_signal(),
                });
                self.publish(&msg.subject(&self.actor_id), &msg.into())?;
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

fn log_debug(client: &ControllerClient, msg: &str) {
    debug(
        client,
        String::from(msg),
        &format!("controller.{}", &client.id),
    );
}

fn log_info(client: &ControllerClient, msg: &str) {
    info(
        client,
        String::from(msg),
        &format!("controller.{}", &client.id),
    );
}

fn log_error(client: &ControllerClient, msg: &str) {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControllerPubMsg {
    #[serde(rename = "set_signal")]
    SetSignal(SignalMsg),
    #[serde(rename = "status")]
    Status {
        id: ClientId,
        timestamp: TimeStamp,
        target: f32,
        #[serde(rename = "type")]
        type_: ControllerType,
    },
}

impl ControllerPubMsg {
    pub fn subject(&self, msg_id: &ClientId) -> Subject {
        match self {
            ControllerPubMsg::SetSignal(SignalMsg {
                id: _,
                timestamp: _,
                signal: _,
            }) => Subject(format!("actor.{}.set_signal", msg_id)),
            ControllerPubMsg::Status {
                id,
                timestamp: _,
                target: _,
                type_: _,
            } => Subject(format!("controller.{}.status", id)),
        }
    }
}

impl Into<PubSubMsg> for ControllerPubMsg {
    fn into(self) -> PubSubMsg {
        PubSubMsg(serde_json::to_string(&self).expect("Pub sub serialization error"))
    }
}
