// pub mod cool_ds18b20;
pub mod cpu_temp;
pub mod ds18b20;
pub mod dummy;
use crate::pub_sub::{
    nats_client::NatsClient, nats_client::NatsConfig, Message, PubSubClient, PubSubError, Subject,
};
use nats::Subscription;
use std::thread::sleep;
use std::time::Duration;

use std::error as std_error;
use std::sync;

pub enum SensorType {
    Dummy,
    DSB,
    RbpiCPU,
    UnknownSensor,
}

impl SensorType {
    pub fn from_str(sensor_type: String) -> Self {
        sensor_type.to_ascii_lowercase();
        match sensor_type.as_ref() {
            "dummy" => Self::Dummy,
            "dsb" => Self::DSB,
            "rbpicpu" => Self::RbpiCPU,
            _ => Self::UnknownSensor,
        }
    }
}

pub type SensorHandle = sync::Arc<sync::Mutex<dyn Sensor>>;

pub fn get_measurement(sensor_mut: &SensorHandle) -> Result<f32, Error> {
    let mut sensor = match sensor_mut.lock() {
        Ok(sensor) => sensor,
        Err(err) => {
            return Err(Error::ThreadLockError(err.to_string()));
        }
    };

    match sensor.get_measurement() {
        Ok(measurement) => Ok(measurement),
        Err(err) => Err(err),
    }
}

pub fn get_id(sensor_mut: &SensorHandle) -> Result<String, Error> {
    match sensor_mut.lock() {
        Ok(sensor) => Ok(sensor.get_id()),
        Err(err) => Err(Error::ThreadLockError(err.to_string())),
    }
}

pub trait Sensor: Send {
    // TODO: it's nice to have this return a common sensor error,
    // but this might snowball when more sensors are added.
    // This should be more generic
    fn get_measurement(&self) -> Result<f32, Error>;
    fn get_id(&self) -> String;
}

pub struct PubSubSensor<S>
where
    S: Sensor,
{
    id: String,
    sensor: S,
    /// TODO: Make generic over PubSubClient
    client: NatsClient,
}

impl<S> PubSubSensor<S>
where
    S: Sensor,
{
    pub fn new(id: &str, sensor: S, config: &NatsConfig) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        PubSubSensor {
            id: id.into(),
            sensor,
            client,
        }
    }

    fn gen_meas_msg(&self, meas: f32) -> Message {
        Message(format!("data: {}", meas))
    }

    fn gen_meas_subject(&self) -> Subject {
        Subject(format!("sensor.{}.measurement", self.id))
    }
}

impl<S> PubSubClient for PubSubSensor<S>
where
    S: Sensor,
{
    fn start_loop(self) -> Result<(), PubSubError> {
        println!("Starting sensor");
        let subject = Subject("command".into());
        let sub = self.subscribe(&subject);
        let client = self.client.clone();
        let mut sensor = self.sensor;
        let id = self.id.clone();
        loop {
            println!("Loop");
            for msg in sub.try_iter() {
                println!("Received a {}", msg);
            }
            println!("Past msg's");
            let meas = sensor.get_measurement()?;
            println!("Got meas");
            client.publish(
                &Subject(format!("sensor.{}.measurement", id)),
                &Message(format!("data: {}", meas)),
            );
            println!("Publishing meas: {}", meas);
            sleep(Duration::from_millis(500));
        }
    }

    fn subscribe(&self, subject: &Subject) -> Subscription {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &Message) {
        todo!();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidAddressStart(String),
    InvalidAddressLength(usize),
    FileReadError(String),
    FileParseError(String),
    ThreadLockError(String),
    InvalidParam(String),
    UnknownSensor(String),
}

impl From<Error> for PubSubError {
    fn from(x: Error) -> PubSubError {
        PubSubError::Generic("Sensor error".into())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidAddressStart(address) => {
                write!(f, "Address must start with 28, got {}", address)
            }
            Error::InvalidAddressLength(address_length) => {
                write!(f, "Address length must be 13, got {}", address_length)
            }
            Error::FileReadError(io_message) => {
                write!(f, "Unable to read from file: {}", io_message)
            }
            Error::FileParseError(measurement) => {
                write!(f, "Could not parse value: {}", measurement)
            }
            Error::ThreadLockError(err) => write!(f, "Unable to acquire sensor lock: {}", err),
            Error::InvalidParam(err) => write!(f, "Invalid sensor param: {}", err),
            Error::UnknownSensor(err) => write!(f, "Unknown sensor: {}", err),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidAddressStart(_) => "Address must start with 28",
            Error::InvalidAddressLength(_) => "Address length must be 13",
            Error::FileReadError(_) => "File read error",
            Error::FileParseError(_) => "File parse error",
            Error::ThreadLockError(_) => "Thread lock error",
            Error::InvalidParam(_) => "Invalid param error",
            Error::UnknownSensor(_) => "Unknown sensor",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
