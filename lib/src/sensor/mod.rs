// pub mod cool_ds18b20;
pub mod cpu_temp;
pub mod ds18b20;
pub mod dummy;
use crate::pub_sub::{
    nats_client::NatsClient, nats_client::NatsConfig, ClientId, Message, PubSubClient, PubSubError,
    Subject,
};
use nats::Subscription;
use std::error as std_error;
use std::thread::sleep;
use std::time::Duration;

pub trait Sensor: Send {
    // TODO: it's nice to have this return a common sensor error,
    // but this might snowball when more sensors are added.
    // This should be more generic
    fn get_measurement(&self) -> Result<f32, Error>;
    fn get_id(&self) -> String;
}

pub struct SensorClient<S>
where
    S: Sensor,
{
    id: ClientId,
    sensor: S,
    /// TODO: Make generic over PubSubClient
    client: NatsClient,
}

impl<S> SensorClient<S>
where
    S: Sensor,
{
    pub fn new(id: ClientId, sensor: S, config: &NatsConfig) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        SensorClient { id, sensor, client }
    }

    fn gen_meas_msg(&self, meas: f32) -> Message {
        Message(format!("{}", meas))
    }

    fn gen_meas_subject(&self) -> Subject {
        Subject(format!("sensor.{}.measurement", self.id))
    }
}

impl<S> PubSubClient for SensorClient<S>
where
    S: Sensor,
{
    fn client_loop(self) -> Result<(), PubSubError> {
        let subject = Subject(format!("command.sensor.{}", self.id));
        let sub = self.subscribe(&subject)?;
        loop {
            for _msg in sub.try_iter() {}
            let meas = self.sensor.get_measurement()?;
            self.publish(&self.gen_meas_subject(), &self.gen_meas_msg(meas))?;
            sleep(Duration::from_millis(5000));
        }
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &Message) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}

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
