use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::ops::Range;

use super::event::Event;
use super::instant::Instant;
use super::DTU;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Timeline {
    pub(crate) events: Vec<Option<Event>>,
    pub(crate) start_points: BTreeSet<Instant>,
    pub(crate) end_points: BTreeSet<Instant>,
    // TODO:  Implement event repetitions
    // weekly_repetitions: Vec<usize>,
    // biweekly_repetitions: Vec<usize>,
    // monthly_repetitions: Vec<usize>,
    // yearly_repetitions: Vec<usize>,
}
impl Timeline {
    /// Creates a new empty timeline
    pub fn new() -> Self {
        Self {
            events: Default::default(),
            start_points: Default::default(),
            end_points: Default::default(),
        }
    }

    /// Creates a new event with a given time range, title, and associated node id to refer to
    ///
    /// Returns the event's id
    pub fn create_event(&mut self, range: Range<DTU>, title: String, node: usize) -> usize {
        let id = self.events.len();
        self.events
            .push(Some(Event::new(range.clone(), title, node)));

        // Instants use their index in the internal events vec as their id
        let Range { start, end } = range;
        self.start_points.insert(Instant::new(start.to_owned(), id));
        self.end_points.insert(Instant::new(end.to_owned(), id));

        id
    }

    /// Deletes an event from the internal vec.
    /// In reality it only replaces it with a [None].
    /// To properly remove all [None] instances in the internal vec, check out [clean][Self::clean]
    pub fn delete_event(&mut self, id: usize) {
        if self.events[id].is_none() {
            return;
        }

        let event = self.events[id].as_ref().unwrap();

        let Range { start, end } = &event.bounds;
        self.start_points
            .remove(&Instant::new(start.to_owned(), id));
        self.end_points.remove(&Instant::new(end.to_owned(), id));

        self.events[id] = None;
    }

    /// Cleans up [None] entries in the internal vec
    ///
    /// The update function's args:
    /// usize - associated node's id
    /// (usize, usize) - map from old index to new index
    ///
    /// Also check out [Graph::clean][crate::graph::Graph::clean]
    /// and [Vec::retain]
    pub fn clean<F>(&mut self, update: F)
    where
        F: Fn(usize, (usize, usize)),
    {
        let mut new_id = 0;
        for old_id in 0..self.events.len() {
            if self.events[old_id].is_some() {
                update(self.events[old_id].as_ref().unwrap().node, (old_id, new_id));
                self.events[new_id] = self.events[old_id].take();
                new_id += 1;
            }
        }
        self.events.truncate(new_id + 1);
    }

    pub fn get_all(&self) -> impl Iterator<Item = &Event> {
        self.events.iter().filter_map(|e| e.as_ref())
    }

    /// Returns all events which are entirely contained within the given time range.
    /// I.e. events always start and end within the given time range.
    ///
    /// # NOTE
    /// The given time range is treated as a closed interval,
    /// NOT an open interval.
    pub fn contains(&self, range: Range<DTU>) -> Option<impl Iterator<Item = &Event>> {
        let Range { start, end } = range;
        let x = self
            .start_points
            .range(start..=end)
            .filter_map(|i| self.events[i.id].as_ref())
            .filter(move |e| e.bounds.end <= end);

        Some(x)
    }

    /// Returns all events which intersect with the given time range.
    /// I.e. events may not be entirely contained within the given time range and
    /// extend before or after the given time range.
    ///
    /// For events that are entirely contained within a time range, see [contains][Self::contains]
    /// Also if possible, always use [contains][Self::contains] instead as
    /// [intersect][Self::intersect] may end up checking unnecessarily more events.
    /// (This may change in the future when a better implementation is found)
    ///
    /// # NOTE
    /// The given time range is treated as a closed interval,
    /// NOT an open interval.
    pub fn intersect(&self, range: Range<DTU>) -> Option<impl Iterator<Item = &Event>> {
        let Range { start, end } = range;
        let x = self
            .end_points
            .range(start..)
            .filter_map(|i| self.events[i.id].as_ref())
            .filter(move |e| e.bounds.start <= end);
        Some(x)
    }
}
impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeDelta, Utc};

    use super::*;

    #[test]
    fn delete() {
        let mut timeline = Timeline::new();
        let now = Utc::now();
        let a = timeline.create_event(now..(now + TimeDelta::seconds(1)), "A".to_string(), 0);
        let _b = timeline.create_event(now..(now + TimeDelta::seconds(2)), "B".to_string(), 1);
        let c = timeline.create_event(
            (now + TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "C".to_string(),
            2,
        );
        let _d = timeline.create_event(now..(now + TimeDelta::seconds(1)), "D".to_string(), 3);
        let _e = timeline.create_event(
            (now - TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "E".to_string(),
            4,
        );

        assert_eq!(timeline.get_all().count(), 5);

        timeline.delete_event(a);
        assert_eq!(timeline.get_all().count(), 4);
        assert_eq!(
            timeline
                .contains(now..(now + TimeDelta::seconds(1)))
                .unwrap()
                .count(),
            1
        );

        timeline.delete_event(c);
        assert_eq!(timeline.get_all().count(), 3);
        assert_eq!(
            timeline
                .contains((now + TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)))
                .unwrap()
                .count(),
            0
        );
    }

    #[test]
    fn contains_multi_short() {
        let mut timeline = Timeline::new();
        let now = Utc::now();
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "A".to_string(), 0);
        timeline.create_event(now..(now + TimeDelta::seconds(2)), "B".to_string(), 1);
        timeline.create_event(
            (now + TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "C".to_string(),
            2,
        );
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "D".to_string(), 3);
        timeline.create_event(
            (now - TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "E".to_string(),
            4,
        );

        let mut iter = timeline
            .contains(now..(now + TimeDelta::seconds(1)))
            .unwrap();

        let x = iter.next().unwrap();
        assert_eq!(x.title, "A");

        let x = iter.next().unwrap();
        assert_eq!(x.title, "D");

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn contains_multi_medium() {
        let mut timeline = Timeline::new();
        let now = Utc::now();
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "A".to_string(), 0);
        timeline.create_event(now..(now + TimeDelta::seconds(2)), "B".to_string(), 1);
        timeline.create_event(
            (now + TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "C".to_string(),
            2,
        );
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "D".to_string(), 3);
        timeline.create_event(
            (now - TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "E".to_string(),
            4,
        );

        let events: Vec<_> = timeline
            .contains(now..(now + TimeDelta::seconds(2)))
            .unwrap()
            .collect();
        assert_eq!(events.len(), 4);
        assert!(events.iter().find(|e| e.title == "A").is_some());
        assert!(events.iter().find(|e| e.title == "B").is_some());
        assert!(events.iter().find(|e| e.title == "C").is_some());
        assert!(events.iter().find(|e| e.title == "D").is_some());
    }

    #[test]
    fn contains_multi_long() {
        let mut timeline = Timeline::new();
        let now = Utc::now();
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "A".to_string(), 0);
        timeline.create_event(now..(now + TimeDelta::seconds(2)), "B".to_string(), 1);
        timeline.create_event(
            (now + TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "C".to_string(),
            2,
        );
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "D".to_string(), 3);
        timeline.create_event(
            (now - TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "E".to_string(),
            4,
        );

        let events: Vec<_> = timeline
            .contains((now - TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)))
            .unwrap()
            .collect();
        assert_eq!(events.len(), 5);
        assert!(events.iter().find(|e| e.title == "A").is_some());
        assert!(events.iter().find(|e| e.title == "B").is_some());
        assert!(events.iter().find(|e| e.title == "C").is_some());
        assert!(events.iter().find(|e| e.title == "D").is_some());
        assert!(events.iter().find(|e| e.title == "E").is_some());
    }

    #[test]
    fn contains_single() {
        let mut timeline = Timeline::new();
        let now = Utc::now();
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "A".to_string(), 0);
        timeline.create_event(now..(now + TimeDelta::seconds(2)), "B".to_string(), 1);
        timeline.create_event(
            (now + TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "C".to_string(),
            2,
        );
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "D".to_string(), 3);
        timeline.create_event(
            (now - TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "E".to_string(),
            4,
        );

        let events: Vec<_> = timeline
            .contains((now + TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)))
            .unwrap()
            .collect();

        assert_eq!(events.len(), 1);
        assert!(events.iter().find(|e| e.title == "C").is_some());
    }

    #[test]
    fn contains_none() {
        let mut timeline = Timeline::new();
        let now = Utc::now();
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "A".to_string(), 0);
        timeline.create_event(now..(now + TimeDelta::seconds(2)), "B".to_string(), 1);
        timeline.create_event(
            (now + TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "C".to_string(),
            2,
        );
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "D".to_string(), 3);
        timeline.create_event(
            (now - TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "E".to_string(),
            4,
        );

        let events: Vec<_> = timeline
            .contains((now + TimeDelta::seconds(2))..(now + TimeDelta::seconds(3)))
            .unwrap()
            .collect();

        assert_eq!(events.len(), 0);
    }

    #[test]
    fn intersect_more() {
        let mut timeline = Timeline::new();
        let now = Utc::now();
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "A".to_string(), 0);
        timeline.create_event(now..(now + TimeDelta::seconds(2)), "B".to_string(), 1);
        timeline.create_event(
            (now + TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "C".to_string(),
            2,
        );
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "D".to_string(), 3);
        timeline.create_event(
            (now - TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "E".to_string(),
            4,
        );

        let events: Vec<_> = timeline
            .intersect(now..(now + TimeDelta::seconds(1)))
            .unwrap()
            .collect();
        assert_eq!(events.len(), 5);
        assert!(events.iter().find(|e| e.title == "A").is_some());
        assert!(events.iter().find(|e| e.title == "B").is_some());
        assert!(events.iter().find(|e| e.title == "C").is_some());
        assert!(events.iter().find(|e| e.title == "D").is_some());
        assert!(events.iter().find(|e| e.title == "E").is_some());
    }

    #[test]
    fn intersect_less() {
        let mut timeline = Timeline::new();
        let now = Utc::now();
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "A".to_string(), 0);
        timeline.create_event(now..(now + TimeDelta::seconds(2)), "B".to_string(), 1);
        timeline.create_event(
            (now + TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "C".to_string(),
            2,
        );
        timeline.create_event(now..(now + TimeDelta::seconds(1)), "D".to_string(), 3);
        timeline.create_event(
            (now - TimeDelta::seconds(1))..(now + TimeDelta::seconds(2)),
            "E".to_string(),
            4,
        );

        let events: Vec<_> = timeline
            .intersect((now + TimeDelta::seconds(2))..(now + TimeDelta::seconds(3)))
            .unwrap()
            .collect();
        assert_eq!(events.len(), 3);
        assert!(events.iter().find(|e| e.title == "B").is_some());
        assert!(events.iter().find(|e| e.title == "C").is_some());
        assert!(events.iter().find(|e| e.title == "E").is_some());
    }
}
