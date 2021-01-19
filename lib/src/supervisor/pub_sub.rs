use crate::control::ControllerConfig;
use crate::pub_sub::PubSubMsg;
use crate::pub_sub::{
    nats_client::decode_nats_data, ClientId, ClientState, PubSubClient, PubSubError, Subject,
};
use crate::supervisor::{ActiveClientsList, Supervisor};
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

impl PubSubClient for Supervisor {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let subject = Subject("command.>".into());
        let sub = self.subscribe(&subject)?;
        let mut state = ClientState::Active;
        while state == ClientState::Active {
            if let Some(msg) = sub.next() {
                state = match SupervisorSubMsg::try_from(&msg) {
                    Ok(cmd) => match self.process_command(cmd, &msg) {
                        Ok(state) => state,
                        Err(err) => self.handle_err(err),
                    },
                    Err(err) => self.handle_err(err.into()),
                };
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SupervisorSubMsg {
    #[serde(rename = "start_controller")]
    StartController { contr_data: NewContrData },
    #[serde(rename = "switch_controller")]
    SwitchController { contr_data: NewContrData },
    #[serde(rename = "list_active_clients")]
    ListActiveClients,
    #[serde(rename = "stop")]
    Stop,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct NewContrData {
    pub(crate) config: ControllerConfig,
    pub(crate) new_target: f32,
}

impl TryFrom<&Message> for SupervisorSubMsg {
    type Error = PubSubError;
    fn try_from(msg: &Message) -> Result<Self, PubSubError> {
        match msg.subject.as_ref() {
            "command.start_controller" => {
                let contr_data: NewContrData = decode_nats_data(&msg.data)?;
                Ok(SupervisorSubMsg::StartController { contr_data })
            }
            "command.switch_controller" => {
                let contr_data: NewContrData = decode_nats_data(&msg.data)?;
                Ok(SupervisorSubMsg::SwitchController { contr_data })
            }
            "command.list_active_clients" => Ok(SupervisorSubMsg::ListActiveClients),
            _ => {
                let msg: String = decode_nats_data(&msg.data)?;
                Err(PubSubError::MessageParse(format!(
                    "Could not parse '{}' to SupervisorSubMsg",
                    msg
                )))
            }
        }
    }
}

impl SupervisorSubMsg {
    pub fn subject(&self) -> Subject {
        match self {
            SupervisorSubMsg::StartController { control_config: _ } => {
                Subject(String::from("command.start_controller"))
            }
            SupervisorSubMsg::SwitchController { control_config: _ } => {
                Subject(String::from("command.switch_controller"))
            }
            _ => panic!("No"),
        }
    }
}

impl Into<PubSubMsg> for SupervisorSubMsg {
    fn into(self) -> PubSubMsg {
        match &self {
            SupervisorSubMsg::StartController { control_config } => PubSubMsg(
                serde_json::to_string(&control_config)
                    .expect("SupervisorSubMsg serialization error"),
            ),
            SupervisorSubMsg::SwitchController { control_config } => PubSubMsg(
                serde_json::to_string(&control_config)
                    .expect("SupervisorSubMsg serialization error"),
            ),
            _ => todo!(),
        }
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

impl Into<PubSubMsg> for SupervisorPubMsg {
    fn into(self) -> PubSubMsg {
        match &self {
            SupervisorPubMsg::ActiveClients(clients) => PubSubMsg(
                serde_json::to_string(&clients).expect("SupervisorPubMsg serialization error"),
            ),
            SupervisorPubMsg::KillClient { client_id } => PubSubMsg(
                serde_json::to_string(&client_id).expect("SupervisorSubMsg serialization error"),
            ),
        }
    }
}
