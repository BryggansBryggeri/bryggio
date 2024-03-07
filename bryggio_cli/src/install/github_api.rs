use serde::Deserialize;
use std::io;
use thiserror::Error;

#[derive(Deserialize)]
pub(crate) struct ReleaseArchitecture {
    #[serde(rename = "browser_download_url")]
    pub(crate) url: String,
    pub(crate) name: String,
}

#[derive(Deserialize)]
pub(crate) struct Release {
    pub(crate) tag_name: String,
    assets: Vec<ReleaseArchitecture>,
}

impl Release {
    pub(crate) fn urls(&self) -> impl Iterator<Item = &ReleaseArchitecture> {
        self.assets.iter()
    }
}

pub(crate) fn latest_github_release(url: &str) -> Result<Release, GithubError> {
    let response_raw = ureq::get(url)
        .call()
        .map_err(|err| GithubError::Request(Box::new(err)))?
        .into_string()?;
    Ok(serde_json::from_str(&response_raw)?)
}

#[derive(Error, Debug)]
pub enum GithubError {
    #[error("Request failed for url: '{0}'")]
    Request(#[from] Box<ureq::Error>),
    #[error("Failed parsing API response: '{0}'")]
    ResponseParse(#[from] io::Error),
    #[error("Failed deserializing data: {0}")]
    Deserialization(#[from] serde_json::Error),
}
