use serde::Deserialize;

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

pub(crate) fn latest_github_release(url: &str) -> Release {
    let response_raw = ureq::get(url)
        .call()
        .expect("ureq call failed")
        .into_string()
        .unwrap();
    serde_json::from_str(&response_raw).unwrap()
}
