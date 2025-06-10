use std::borrow::Borrow;

use chrono::serde::ts_seconds;
use serde::{Deserialize, Serialize};

use super::DTU;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instant {
    #[serde(with = "ts_seconds")]
    pub time: DTU,

    /// Instants use their index in the internal events vec (see
    /// [Timeline::events][super::timeline::Timeline::events]) as their id
    pub(crate) id: usize,
}
impl Instant {
    pub fn new(time: DTU, id: usize) -> Self {
        Self { time, id }
    }
}
impl PartialEq for Instant {
    fn eq(&self, other: &Self) -> bool {
        self.time.eq(&other.time) && self.id.eq(&other.id)
    }
}
impl Eq for Instant {}
impl Borrow<DTU> for Instant {
    fn borrow(&self) -> &DTU {
        &self.time
    }
}
impl Ord for Instant {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time).then(self.id.cmp(&other.id))
    }
}
impl PartialOrd for Instant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
