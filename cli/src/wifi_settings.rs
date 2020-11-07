use std::str::FromStr;
pub(crate) struct WifiSettings {
    ssid: Ssid,
    password: Password,
}

#[derive(Debug, Clone)]
pub struct Ssid(String);

impl FromStr for Ssid {
    type Err = String;
    fn from_str(x: &str) -> Result<Self, String> {
        Ok(Ssid(String::from(x)))
    }
}

#[derive(Debug, Clone)]
pub enum Password {
    PlainText(String),
    Hash(String),
}

impl From<Password> for String {
    fn from(pass: Password) -> Self {
        match pass {
            Password::PlainText(pass) => pass,
            Password::Hash(pass) => pass,
        }
    }
}

impl FromStr for Password {
    type Err = String;
    fn from_str(x: &str) -> Result<Self, String> {
        Ok(Password::PlainText(String::from(x)))
    }
}

impl WifiSettings {
    pub(crate) fn new(ssid: Ssid, password: Password) -> Self {
        WifiSettings { ssid, password }
    }

    pub(crate) fn to_wpa_supplicant_entry(&self) -> String {
        format!(
            "network={{\n\tssid=\"{}\"\n\tpsk=\"{}\"\n}}",
            self.ssid.0,
            String::from(self.password.clone())
        )
    }
}
