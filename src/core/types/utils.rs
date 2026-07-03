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

impl From<Timestamp> for f64 {
    fn from(value: Timestamp) -> f64 {
        value.0
    }
}

impl From<f64> for Timestamp {
    fn from(n: f64) -> Timestamp {
        Timestamp(n)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(value: Timestamp) -> DateTime<Utc> {
        let (seconds, nanoseconds) = (value.0.floor() as i64, (value.0 % 1. * 1_000_000_000.) as u32);
        DateTime::<Utc>::from_timestamp(seconds, nanoseconds).unwrap()
    }
}