use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DateData {
    pub date: NaiveDate
}

impl DateData {
    // TODO: as suggested, use a u64 for the key of the hashmap to reduce heap access overhead.
    /// Return a formatted string used for storing date nodes in `dates` hashmap of the save file.
    ///
    /// # Returns
    /// A formatted date string.
    pub fn format_for_hashmap(&self) -> String {
        self.date.format("%Y-%m-%d").to_string()
    }
}

pub trait HashMapFormatter {
    fn hashmap_format(&self) -> String;
}

impl HashMapFormatter for NaiveDate {
    fn hashmap_format(&self) -> String {
        self.format("%Y-%m-%d").to_string()
    }
}
