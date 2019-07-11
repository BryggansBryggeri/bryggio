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
