use crate::{ActiveClientsList, SensorBox};
use bryggio_lib::pub_sub::{ClientId, ClientState, PubSubClient, PubSubError, Subject};
use bryggio_lib::pub_sub::{MessageParseError, PubSubMsg};
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

impl PubSubClient for SensorBox {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let subject = Subject("sensor_box.>".into());
        let sub = self.subscribe(&subject)?;
        let mut state = ClientState::Active;
        while state == ClientState::Active {
            if let Some(msg) = sub.next() {
                state = match SensorBoxSubMsg::try_from(&msg) {
                    Ok(cmd) => match self.process_command(cmd, &msg) {
                        Ok(state) => state,
                        Err(err) => self.handle_err(err),
                    },
                    Err(err) => self.handle_err(PubSubError::from(err).into()),
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
    type Error = MessageParseError;
    fn try_from(msg: &Message) -> Result<Self, MessageParseError> {
        match msg.subject.as_ref() {
            "sensor_box.list_active_clients" => Ok(SensorBoxSubMsg::ListActiveClients),
            _ => Err(MessageParseError::InvalidSubject(Subject(
                msg.subject.clone(),
            ))),
        }
    }
}

impl SensorBoxSubMsg {
    pub fn subject(&self) -> Subject {
        todo!()
        // match self {
        //     _ => panic!("No"),
        // }
    }
}

impl From<SensorBoxSubMsg> for PubSubMsg {
    fn from(_msg: SensorBoxSubMsg) -> PubSubMsg {
        todo!()
        // match &msg {
        //     _ => todo!(),
        // }
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
    pub fn subject(&self) -> Subject {
        match self {
            SensorBoxPubMsg::ActiveClients(_) => Subject(String::from("sensor_box.active_clients")),
            SensorBoxPubMsg::KillClient { client_id } => {
                Subject(format!("sensor_box.kill.{}", client_id))
            }
        }
    }
}

impl From<SensorBoxPubMsg> for PubSubMsg {
    fn from(msg: SensorBoxPubMsg) -> PubSubMsg {
        match &msg {
            SensorBoxPubMsg::ActiveClients(clients) => PubSubMsg(
                serde_json::to_string(&clients).expect("SupervisorPubMsg serialization error"),
            ),
            SensorBoxPubMsg::KillClient { client_id } => PubSubMsg(
                serde_json::to_string(&client_id).expect("SupervisorSubMsg serialization error"),
            ),
        }
    }
}
