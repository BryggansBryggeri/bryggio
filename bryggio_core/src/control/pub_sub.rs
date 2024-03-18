use crate::actor::pub_sub::SignalMsg;
use crate::actor::ActorSignal;
use crate::control::Control;
use crate::control::ControllerType;
use crate::logger::{error, info};
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsClientConfig,
    ClientId, PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::sensor::SensorMsg;
use crate::supervisor::pub_sub::SupervisorPubMsg;
use crate::time::{TimeStamp, LOOP_PAUSE_TIME};
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::thread::sleep;

pub struct ControllerClient {
    id: ClientId,
    actor_id: ClientId,
    sensor_id: ClientId,
    controller: Box<dyn Control>,
    client: NatsClient,
    type_: ControllerType,
}

impl PubSubClient for ControllerClient {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let kill_cmd = self.subscribe(
            &SupervisorPubMsg::KillClient {
                client_id: self.id.clone(),
            }
            .subject(),
        )?;
        let contr_sub = self.subscribe(&ControllerSubMsg::subject(&self.id))?;
        let sensor_sub = self.subscribe(&SensorMsg::subject(&self.sensor_id))?;
        log_info(
            &self,
            &format!(
                "Starting contr. client: {}\n\tMode: {:?}\n\tActor: {}\n\tSensor: {}\n\tTarget: {}",
                &self.id,
                &self.type_,
                &self.actor_id,
                &self.sensor_id,
                &self.controller.get_target()
            ),
        );
        self.status_update();
        loop {
            if let Some(msg) = kill_cmd.try_next() {
                // TODO: Proper Status PubMsg.
                let actor_msg = ControllerPubMsg::TurnOffActor;
                let response = match self.client.request(
                    &actor_msg.subject(&self.actor_id),
                    &PubSubMsg("turn_off".into()),
                ) {
                    Ok(_) => format!("{}", self.controller.get_target()),
                    Err(err) => format!("Failed turning actor off {}", err),
                };
                msg.respond(response).map_err(|err| PubSubError::Reply {
                    task: "kill contr.",
                    msg: msg.clone(),
                    source: err,
                })?;
                self.status_update();
                log_info(&self, "Contr. client killed.");
                return Ok(());
            }

            if let Some(nats_msg) = contr_sub.try_next() {
                // TODO: Match and log error
                match ControllerSubMsg::try_from(nats_msg.clone()) {
                    Ok(msg) => match msg {
                        ControllerSubMsg::SetTarget(new_target) => {
                            let response = match self.controller.validate_target(new_target) {
                                Ok(new_target) => {
                                    self.controller.set_target(new_target);
                                    log_info(
                                        &self,
                                        &format!(
                                            "Setting target '{}' for controller '{}'",
                                            new_target, self.id
                                        ),
                                    );
                                    format!(
                                        "Target '{}' set for controller '{}'",
                                        new_target, self.id
                                    )
                                }
                                Err(err) => {
                                    log_error(&self, &err.to_string());
                                    err.to_string()
                                }
                            };
                            nats_msg
                                .respond(response)
                                .map_err(|err| PubSubError::Reply {
                                    task: "List active clients",
                                    msg: nats_msg.clone(),
                                    source: err,
                                })?;
                        }
                    },
                    Err(err) => log_error(&self, &err.to_string()),
                };
                self.status_update();
            }

            if let Some(meas_msg) = sensor_sub.try_next() {
                if let Ok(msg) = SensorMsg::try_from(meas_msg) {
                    self.controller.calculate_signal(msg.meas.ok());
                }
                let msg = ControllerPubMsg::SetActorSignal(SignalMsg {
                    id: self.actor_id.clone(),
                    timestamp: TimeStamp::now(),
                    signal: ActorSignal::new(
                        self.actor_id.clone(),
                        self.controller.get_control_signal(),
                    ),
                });
                self.publish(&msg.subject(&self.actor_id), &msg.into())?;
                self.status_update();
            }
            sleep(LOOP_PAUSE_TIME);
        }
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}

impl ControllerClient {
    pub fn new(
        id: ClientId,
        actor_id: ClientId,
        sensor_id: ClientId,
        controller: Box<dyn Control>,
        config: &NatsClientConfig,
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
            log_error(self, &format!("Could not publish status update: {}", err));
        };
    }
}

// fn log_debug(client: &ControllerClient, msg: &str) {
//     debug(
//         client,
//         String::from(msg),
//         &format!("controller.{}", &client.id),
//     );
// }

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

impl From<ControllerSubMsg> for PubSubMsg {
    fn from(msg: ControllerSubMsg) -> PubSubMsg {
        match msg {
            ControllerSubMsg::SetTarget(new_target) => PubSubMsg(new_target.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControllerPubMsg {
    #[serde(rename = "set_signal")]
    // TODO: Remove and use actor?
    SetActorSignal(SignalMsg),
    #[serde(rename = "turn_off")]
    TurnOffActor,
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
            ControllerPubMsg::SetActorSignal(SignalMsg {
                id: _,
                timestamp: _,
                signal: _,
            }) => Subject(format!("actor.{}.set_signal", msg_id)),
            ControllerPubMsg::TurnOffActor => Subject(format!("actor.{}.turn_off", msg_id)),
            ControllerPubMsg::Status {
                id,
                timestamp: _,
                target: _,
                type_: _,
            } => Subject(format!("controller.{}.status", id)),
        }
    }
}

impl From<ControllerPubMsg> for PubSubMsg {
    fn from(msg: ControllerPubMsg) -> PubSubMsg {
        PubSubMsg(serde_json::to_string(&msg).expect("Pub sub serialization error"))
    }
}
