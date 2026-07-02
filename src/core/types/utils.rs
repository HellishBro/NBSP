use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, PartialOrd, Debug)]
#[serde(transparent)]
pub struct Timestamp(f64);

impl Timestamp {
    pub fn now() -> Timestamp {
        Timestamp(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64())
    }
}

impl Into<f64> for Timestamp {
    fn into(self) -> f64 {
        self.0
    }
}

impl From<f64> for Timestamp {
    fn from(n: f64) -> Timestamp {
        Timestamp(n)
    }
}

impl Into<DateTime<Utc>> for Timestamp {
    fn into(self) -> DateTime<Utc> {
        let (seconds, nanoseconds) = (self.0.floor() as i64, (self.0 % 1. * 1_000_000_000.) as u32);
        DateTime::from_timestamp(seconds, nanoseconds).unwrap()
    }
}