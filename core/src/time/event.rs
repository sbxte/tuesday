use std::borrow::Borrow;
use std::ops::Range;

use serde::{Deserialize, Serialize};

use super::DTU;

// TODO: Implement event repetitions
// TODO: How should leap days work with this?
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub enum EventRepetition {
    /// Happens once and only once
    #[default]
    Once,
    /// Repeats on the same day every week
    /// e.g. every Monday
    Weekly,
    /// Repeats on the same day every 2 weeks
    Biweekly,
    /// Repeats on the same date every month,
    /// e.g. 5th July, 5th August, 5th September, etc.
    Monthly,
    /// Repeats on the same date every year,
    /// e.g. 5th July 2024, 5th July 2025, 5th July 2026, etc.
    Yearly,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Event {
    pub bounds: Range<DTU>,

    pub title: String,

    pub node: usize,
    //
    // pub repetition: EventRepetition,
    // /// Is [Some] when [EventRepetition] is not [EventRepetition::Once].
    // /// Stores a repetition id for the repetition system
    // repetition_id: Option<usize>,
}
impl Event {
    pub fn new(bounds: Range<DTU>, title: String, node: usize) -> Self {
        Self {
            bounds,
            title,
            node,
        }
    }
}
impl Borrow<DTU> for Event {
    fn borrow(&self) -> &DTU {
        &self.bounds.start
    }
}
impl Borrow<Range<DTU>> for Event {
    fn borrow(&self) -> &Range<DTU> {
        &self.bounds
    }
}
impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bounds
            .start
            .cmp(&other.bounds.start)
            .then(self.bounds.end.cmp(&other.bounds.end))
            .then(self.node.cmp(&other.node))
    }
}
impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
