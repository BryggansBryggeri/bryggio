use crate::{ActiveClientsList, SensorBox};
use bryggio_lib::pub_sub::PubSubMsg;
use bryggio_lib::pub_sub::{
    nats_client::decode_nats_data, ClientId, ClientState, PubSubClient, PubSubError, Subject,
};
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

impl PubSubClient for SensorBox {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let subject = Subject("command.>".into());
        let sub = self.subscribe(&subject)?;
        let mut state = ClientState::Active;
        while state == ClientState::Active {
            if let Some(msg) = sub.next() {
                state = match SensorBoxSubMsg::try_from(&msg) {
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
pub enum SensorBoxSubMsg {
    #[serde(rename = "list_active_clients")]
    ListActiveClients,
    #[serde(rename = "stop")]
    Stop,
}

impl TryFrom<&Message> for SensorBoxSubMsg {
    type Error = PubSubError;
    fn try_from(msg: &Message) -> Result<Self, PubSubError> {
        match msg.subject.as_ref() {
            "command.list_active_clients" => Ok(SensorBoxSubMsg::ListActiveClients),
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

impl SensorBoxSubMsg {
    pub fn subject(&self) -> Subject {
        match self {
            _ => panic!("No"),
        }
    }
}

impl Into<PubSubMsg> for SensorBoxSubMsg {
    fn into(self) -> PubSubMsg {
        match &self {
            _ => todo!(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SensorBoxPubMsg {
    #[serde(rename = "active_clients")]
    ActiveClients(ActiveClientsList),
    #[serde(rename = "kill_client")]
    KillClient { client_id: ClientId },
}

impl SensorBoxPubMsg {
    pub(crate) fn subject(&self) -> Subject {
        match self {
            SensorBoxPubMsg::ActiveClients(_) => Subject(String::from("supervisor.active_clients")),
            SensorBoxPubMsg::KillClient { client_id } => {
                Subject(format!("supervisor.kill.{}", client_id))
            }
        }
    }
}

impl Into<PubSubMsg> for SensorBoxPubMsg {
    fn into(self) -> PubSubMsg {
        match &self {
            SensorBoxPubMsg::ActiveClients(clients) => PubSubMsg(
                serde_json::to_string(&clients).expect("SupervisorPubMsg serialization error"),
            ),
            SensorBoxPubMsg::KillClient { client_id } => PubSubMsg(
                serde_json::to_string(&client_id).expect("SupervisorSubMsg serialization error"),
            ),
        }
    }
}
