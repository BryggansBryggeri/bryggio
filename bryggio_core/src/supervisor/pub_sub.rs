use crate::pub_sub::PubSubMsg;
use crate::pub_sub::{
    nats_client::decode_nats_data, ClientId, ClientState, PubSubClient, PubSubError, Subject,
};
use crate::supervisor::{ActiveClientsList, Supervisor};
use crate::{control::ControllerConfig, pub_sub::MessageParseError};
use async_nats::{Message, Subscriber};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use super::SupervisorError;

impl PubSubClient for Supervisor {
    async fn client_loop(mut self) -> Result<(), PubSubError> {
        let subject = Subject("command.>".into());
        let mut sub = self.subscribe(&subject).await?;
        let mut state = ClientState::Active;
        while state == ClientState::Active {
            if let Some(msg) = sub.next().await {
                println!("sup msg {:?}", msg);
                state = match SupervisorSubMsg::try_from(&msg) {
                    Ok(cmd) => match self.process_command(cmd, &msg).await {
                        Ok(state) => state,
                        Err(err) => self.handle_err(err).await,
                    },
                    Err(err) => {
                        if let Some(reply_subj) = msg.reply {
                            self.publish(&reply_subj.into(), &PubSubMsg(err.to_string()))
                                .await?;
                        }
                        self.handle_err(SupervisorError::from(PubSubError::from(err)))
                            .await
                    }
                };
            }
        }
        Ok(())
    }

    async fn subscribe(&self, subject: &Subject) -> Result<Subscriber, PubSubError> {
        self.client.subscribe(subject).await
    }

    async fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg).await
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SupervisorSubMsg {
    #[serde(rename = "start_controller")]
    StartController { contr_data: NewContrData },
    #[serde(rename = "stop_controller")]
    StopController { contr_id: ClientId },
    #[serde(rename = "switch_controller")]
    SwitchController { contr_data: NewContrData },
    #[serde(rename = "list_active_clients")]
    ListActiveClients,
    #[serde(rename = "stop")]
    Stop,
}

impl TryFrom<&Message> for SupervisorSubMsg {
    type Error = MessageParseError;
    fn try_from(msg: &Message) -> Result<Self, MessageParseError> {
        match msg.subject.as_ref() {
            "command.start_controller" => {
                let contr_data: NewContrData = decode_nats_data(&msg.payload)?;
                Ok(SupervisorSubMsg::StartController { contr_data })
            }
            "command.stop_controller" => {
                let contr_id: ClientId = decode_nats_data(&msg.payload)?;
                Ok(SupervisorSubMsg::StopController { contr_id })
            }
            "command.switch_controller" => {
                let contr_data: NewContrData = decode_nats_data(&msg.payload)?;
                Ok(SupervisorSubMsg::SwitchController { contr_data })
            }
            "command.list_active_clients" => Ok(SupervisorSubMsg::ListActiveClients),
            "command.stop" => Ok(SupervisorSubMsg::Stop),
            _ => Err(MessageParseError::InvalidSubject(Subject(
                msg.subject.to_string(),
            ))),
        }
    }
}

impl SupervisorSubMsg {
    pub fn subject(&self) -> Subject {
        match self {
            SupervisorSubMsg::StartController { contr_data: _ } => {
                Subject::from("command.start_controller")
            }
            SupervisorSubMsg::SwitchController { contr_data: _ } => {
                Subject::from("command.switch_controller")
            }
            SupervisorSubMsg::StopController { contr_id: _ } => {
                Subject::from("command.stop_controller")
            }
            SupervisorSubMsg::ListActiveClients => Subject::from("command.list_active_clients"),
            SupervisorSubMsg::Stop => Subject::from("command.stop"),
        }
    }
}

impl From<SupervisorSubMsg> for PubSubMsg {
    fn from(msg: SupervisorSubMsg) -> PubSubMsg {
        match &msg {
            SupervisorSubMsg::StartController { contr_data } => PubSubMsg(
                serde_json::to_string(&contr_data).expect("SupervisorSubMsg serialization error"),
            ),
            SupervisorSubMsg::SwitchController { contr_data } => PubSubMsg(
                serde_json::to_string(&contr_data).expect("SupervisorSubMsg serialization error"),
            ),
            // Empty message
            SupervisorSubMsg::ListActiveClients => PubSubMsg("".into()),
            SupervisorSubMsg::Stop => PubSubMsg("".into()),
            _ => todo!(
                "Supervisor command '{:?}' not implemented yet",
                msg.subject()
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewContrData {
    pub(crate) config: ControllerConfig,
    pub(crate) new_target: f32,
}

impl NewContrData {
    pub fn new(config: ControllerConfig, new_target: f32) -> Self {
        NewContrData { config, new_target }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SupervisorPubMsg {
    #[serde(rename = "active_clients")]
    ActiveClients(ActiveClientsList),
    #[serde(rename = "kill_client")]
    KillClient { client_id: ClientId },
}

impl SupervisorPubMsg {
    pub(crate) fn subject(&self) -> Subject {
        match self {
            SupervisorPubMsg::ActiveClients(_) => {
                Subject(String::from("supervisor.active_clients"))
            }
            SupervisorPubMsg::KillClient { client_id } => {
                Subject(format!("supervisor.kill.{}", client_id))
            }
        }
    }
}

impl From<SupervisorPubMsg> for PubSubMsg {
    fn from(msg: SupervisorPubMsg) -> PubSubMsg {
        match &msg {
            SupervisorPubMsg::ActiveClients(clients) => PubSubMsg(
                serde_json::to_string(&clients).expect("SupervisorPubMsg serialization error"),
            ),
            SupervisorPubMsg::KillClient { client_id } => PubSubMsg(
                serde_json::to_string(&client_id).expect("SupervisorSubMsg serialization error"),
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::control::ControllerType;

    #[test]
    fn test_new_contr_data_parse() {
        let text = r#"{"config":{"controller_id":"mash","actor_id":"mash_heater","sensor_id":"mash_temp","type":{"hysteresis":{"offset_on":10,"offset_off":5}}},"new_target":16.0}"#;
        let parsed: NewContrData = serde_json::from_str(text).unwrap();
        let true_ = NewContrData {
            config: ControllerConfig {
                controller_id: ClientId(String::from("mash")),
                actor_id: ClientId(String::from("mash_heater")),
                sensor_id: ClientId(String::from("mash_temp")),
                type_: ControllerType::Hysteresis {
                    offset_on: 10.0,
                    offset_off: 5.0,
                },
            },
            new_target: 16.0,
        };
        // TODO: assert_eq!(parsed, true_);
    }
}
