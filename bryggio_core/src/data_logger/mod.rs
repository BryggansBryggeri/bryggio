//! Data logger
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::thread::sleep;

use crate::actor::pub_sub::{actor_current_signal_subject, ActorPubMsg, SignalMsg};
use crate::pub_sub::ClientId;
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsClientConfig,
    PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::sensor::SensorMsg;
use crate::time::{TimeStamp, LOOP_PAUSE_TIME};
use csv::WriterBuilder;
use nats::Subscription;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

pub struct DataLogger {
    id: ClientId,
    client: NatsClient,
    log_file_path: PathBuf,
}

impl PubSubClient for DataLogger {
    fn client_loop(self) -> Result<(), PubSubError> {
        // let mut log_file = OpenOptions::new()
        //     .write(true)
        //     .append(true)
        //     .create(true)
        //     .open(&self.log_file_path)?;
        // writeln!(log_file, "{}", Record::headers())?;

        let mut wtr = WriterBuilder::new()
            // .has_headers(false)
            .from_path(&self.log_file_path)
            .expect("Could not create CSV writer.");
        let wildcard_id = ClientId::from("*");
        let sensor_sub = self.subscribe(&Subject(String::from("sensor.*.measurement")))?;
        let actor_sub = self.subscribe(&actor_current_signal_subject(&wildcard_id))?;
        loop {
            if let Some(msg) = sensor_sub.try_next() {
                let data = decode_nats_data::<SensorMsg>(&msg.data).expect("Failed decoding data");
                let rec_str = Record::from(data);
                wtr.serialize(rec_str).expect("Failed serialising");
                wtr.flush().expect("Failed flushing");
            }
            if let Some(msg) = actor_sub.try_next() {
                let data = decode_nats_data::<SignalMsg>(&msg.data).expect("Failed decoding data");
                let rec_str = Record::from(data);
                wtr.serialize(rec_str).expect("Failed serialising");
                wtr.flush().expect("Failed flushing");
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

impl DataLogger {
    pub fn new(id: ClientId, config: &NatsClientConfig, log_file_path: PathBuf) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        Self {
            id,
            client,
            log_file_path,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    id: ClientId,
    local_ts: TimeStamp,
    ext_ts: TimeStamp,
    value: Value,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Val(f32),
    Err(String),
}

impl Record {
    pub fn new(id: ClientId, ext_ts: TimeStamp, value: Value) -> Self {
        Self {
            id,
            local_ts: TimeStamp::now(),
            ext_ts,
            value,
        }
    }

    pub fn headers() -> &'static str {
        "id,local_ts,ext_ts,value"
    }
}

impl From<SensorMsg> for Record {
    fn from(x: SensorMsg) -> Self {
        let val = match x.meas {
            Ok(val) => Value::Val(val),
            Err(err) => Value::Err(err.to_string()),
        };
        Record::new(x.id, x.timestamp, val)
    }
}

impl From<SignalMsg> for Record {
    fn from(x: SignalMsg) -> Self {
        Record::new(x.signal.id, x.timestamp, Value::Val(x.signal.signal))
    }
}

#[cfg(test)]
mod test {
    use csv::{ReaderBuilder, Writer};

    use super::*;

    #[test]
    fn deserialise_ok() {
        let true_rec = Record::new(
            ClientId::from("test"),
            TimeStamp(15141214),
            Value::Val(32.1),
        );
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .from_reader("test,1635863358217,15141214,32.1".as_bytes());
        let mut iter = rdr.deserialize();
        let parsed_rec: Record = iter.next().unwrap().unwrap();
        assert_eq!(true_rec.id, parsed_rec.id);
        assert_eq!(true_rec.ext_ts, parsed_rec.ext_ts);
        assert_eq!(true_rec.value, parsed_rec.value);
    }

    #[test]
    fn deserialise_err() {
        let true_rec = Record::new(
            ClientId::from("test"),
            TimeStamp(15141214),
            Value::Err(String::from("Failure")),
        );
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .from_reader("test,1635863358217,15141214,Failure".as_bytes());
        let mut iter = rdr.deserialize();
        let parsed_rec: Record = iter.next().unwrap().unwrap();
        assert_eq!(true_rec.id, parsed_rec.id);
        assert_eq!(true_rec.ext_ts, parsed_rec.ext_ts);
        assert_eq!(true_rec.value, parsed_rec.value);
    }

    #[test]
    fn serialise() {
        let rec = Record::new(
            ClientId::from("test"),
            TimeStamp(15141214),
            Value::Val(32.1),
        );
        let mut wtr = Writer::from_writer(vec![]);
        wtr.serialize(rec).unwrap();
        String::from_utf8(wtr.into_inner().unwrap()).unwrap();
    }
}
