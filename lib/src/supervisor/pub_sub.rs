use crate::control::ControllerConfig;
use crate::pub_sub::PubSubMsg;
use crate::pub_sub::{
    nats_client::decode_nats_data, ClientId, ClientState, PubSubClient, PubSubError, Subject,
};
use crate::supervisor::Supervisor;
use nats::{Message, Subscription};
use serde::Deserialize;
use std::convert::TryFrom;
pub(crate) const SUPERVISOR_SUBJECT: &str = "supervisor";

#[derive(Deserialize)]
pub enum SupervisorSubMsg {
    #[serde(rename = "start_controller")]
    StartController { control_config: ControllerConfig },
    #[serde(rename = "switch_controller")]
    SwitchController { control_config: ControllerConfig },
    #[serde(rename = "list_active_clients")]
    ListActiveClients,
    #[serde(rename = "kill_client")]
    KillClient { client_id: ClientId },
    #[serde(rename = "stop")]
    Stop,
}

impl TryFrom<Message> for SupervisorSubMsg {
    type Error = PubSubError;
    fn try_from(msg: Message) -> Result<Self, PubSubError> {
        match msg.subject.as_ref() {
            "command.start_controller" => {
                let control_config: ControllerConfig = decode_nats_data(&msg.data)?;
                Ok(SupervisorSubMsg::StartController { control_config })
            }
            "command.switch_controller" => {
                let control_config: ControllerConfig = decode_nats_data(&msg.data)?;
                Ok(SupervisorSubMsg::SwitchController { control_config })
            }
            "command.list_active_clients" => Ok(SupervisorSubMsg::ListActiveClients),
            _ => {
                let msg: String = decode_nats_data(&msg.data)?;
                Err(PubSubError::MessageParse(msg))
            }
        }
    }
}

impl SupervisorSubMsg {
    pub fn subject(id: &ClientId, cmd: &str) -> Subject {
        Subject(format!("{}.{}.{}", SUPERVISOR_SUBJECT, cmd, id))
    }
}

impl PubSubClient for Supervisor {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let subject = Subject("command.>".into());
        let sub = self.subscribe(&subject)?;
        let mut state = ClientState::Active;
        while state == ClientState::Active {
            if let Some(msg) = sub.next() {
                state = match SupervisorSubMsg::try_from(msg) {
                    Ok(cmd) => match self.process_command(&cmd) {
                        Ok(state) => state,
                        Err(err) => Supervisor::handle_err(err),
                    },
                    Err(err) => Supervisor::handle_err(err.into()),
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
