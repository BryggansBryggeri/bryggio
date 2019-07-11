use std::error;
use std::fmt;

// TODO: Error constructor with msg (in trait perhaps???)
#[derive(Debug, Clone)]
pub struct ParamError;

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid param values.")
    }
}
// This is important for other errors to wrap this one.
impl error::Error for ParamError {
    fn description(&self) -> &str {
        "Invalid param values."
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug, Clone)]
pub struct KeyError;

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Key not in collection.")
    }
}
impl error::Error for KeyError {
    fn description(&self) -> &str {
        "Key not in collection."
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

#[derive(Debug, Clone)]
pub enum SensorError {
    InvalidAddressStart(String),
    InvalidAddressLength(usize),
    FileReadError(String),
    FileParseError(String),
}

impl fmt::Display for SensorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            &SensorError::InvalidAddressStart(address) => {
                write!(f, "Address must start with 28, got {}", address)
            }
            &SensorError::InvalidAddressLength(address_length) => {
                write!(f, "Address length must be 13, got {}", address_length)
            }
            &SensorError::FileReadError(io_message) => {
                write!(f, "Unable to read from file {}", io_message)
            }
            &SensorError::FileParseError(measurement) => {
                write!(f, "Could not parse value: {}", measurement)
            }
        }
    }
}
impl error::Error for SensorError {
    fn description(&self) -> &str {
        match self {
            &SensorError::InvalidAddressStart(_) => "Address must start with 28",
            &SensorError::InvalidAddressLength(_) => "Address length must be 13",
            &SensorError::FileReadError(_) => "File read error",
            &SensorError::FileParseError(_) => "File parse error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
