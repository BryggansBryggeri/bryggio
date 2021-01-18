use serde::Deserialize;
use url::Url;

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
    pub(crate) fn url(&self, predicate: fn(&ReleaseArchitecture) -> bool) -> Url {
        #[cfg(target_os = "linux")]
        let os = "linux-amd64";
        #[cfg(target_os = "macos")]
        let os = "darwin-amd64";
        #[cfg(target_os = "windows")]
        let os = "windows-amd64";
        #[cfg(target_arch = "x86_64")]
        let arch = "amd64";
        #[cfg(target_arch = "arm")]
        let arch = "arm7";
        self.assets
            .iter()
            .filter(|x| x.name.contains(os))
            .filter(|x| x.name.contains(arch))
            .filter(|x| predicate(x))
            .map(|x| Url::parse(&x.url))
            .last()
            .unwrap()
            .unwrap()
    }
}
