use derive_more::{Add, Display, Sub};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(
    Copy, Clone, Debug, Add, Sub, Display, Deserialize, Serialize, Ord, PartialOrd, PartialEq, Eq,
)]
pub struct TimeStamp(pub(crate) u128);

impl TimeStamp {
    pub fn now() -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        TimeStamp(since_the_epoch.as_millis())
    }
}
