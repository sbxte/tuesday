use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DateData {
    pub date: NaiveDate
}
