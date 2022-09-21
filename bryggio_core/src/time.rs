use derive_more::{Add, Display, Sub};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

pub(crate) const LOOP_PAUSE_TIME: Duration = Duration::from_millis(100);

#[cfg(test)]
mod test {
    use super::*;
    use chrono::prelude::*;
    use std::convert::TryFrom;

    #[test]
    fn test_reversibility() {
        use chrono::Utc;
        let dt = Utc.ymd(1988, 10, 25).and_hms(8, 51, 32);
        let ts = TimeStamp(u128::try_from(dt.timestamp()).expect("i64 -> u128 conv failed."));
        let naive = NaiveDateTime::from_timestamp(
            i64::try_from(ts.0).expect("u128 -> i64 conv failed."),
            0,
        );
        let new_dt: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        assert!(dt == new_dt);
    }
}
