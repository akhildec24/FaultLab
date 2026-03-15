//! Deterministic event scheduler.
//!
//! The scheduler maintains a priority queue of events ordered by virtual
//! time. It advances its own virtual clock rather than depending on real
//! time, enabling deterministic replay and faster-than-real-time execution.

use crate::types::*;
use std::collections::BinaryHeap;

/// A scheduled event with its virtual time.
#[derive(Debug, Clone)]
struct ScheduledEvent {
    time: VirtualTime,
    sequence: u64,
    event: Event,
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.sequence == other.sequence
    }
}

impl Eq for ScheduledEvent {}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // BinaryHeap is a max-heap, so we reverse for min-heap behaviour.
        other
            .time
            .cmp(&self.time)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

/// The event scheduler.
pub struct Scheduler {
    queue: BinaryHeap<ScheduledEvent>,
    current_time: VirtualTime,
    sequence_counter: u64,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            current_time: VirtualTime::zero(),
            sequence_counter: 0,
        }
    }

    /// Schedule an event at a specific virtual time.
    pub fn schedule(&mut self, time: VirtualTime, event: Event) {
        let seq = self.sequence_counter;
        self.sequence_counter += 1;
        self.queue.push(ScheduledEvent {
            time,
            sequence: seq,
            event,
        });
    }

    /// Pop the next event in virtual-time order.
    pub fn next_event(&mut self) -> Option<(VirtualTime, Event)> {
        let scheduled = self.queue.pop()?;
        self.current_time = scheduled.time;
        Some((scheduled.time, scheduled.event))
    }

    /// Peek at the next event without popping it.
    pub fn peek(&self) -> Option<(VirtualTime, &Event)> {
        self.queue.peek().map(|e| (e.time, &e.event))
    }

    /// Current virtual time.
    pub fn now(&self) -> VirtualTime {
        self.current_time
    }

    /// Number of pending events.
    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    /// Whether there are no pending events.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_execute_in_time_order() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule(
            VirtualTime(300),
            Event::RequestCompleted {
                request_id: 1,
                node_id: "db".into(),
                success: true,
            },
        );
        scheduler.schedule(
            VirtualTime(100),
            Event::RequestArrived {
                request_id: 1,
                node_id: "service".into(),
            },
        );
        scheduler.schedule(
            VirtualTime(200),
            Event::RequestStarted {
                request_id: 1,
                node_id: "service".into(),
            },
        );

        let times: Vec<u64> = std::iter::from_fn(|| scheduler.next_event())
            .map(|(t, _)| t.millis())
            .collect();

        assert_eq!(times, vec![100, 200, 300]);
    }

    #[test]
    fn same_time_events_execute_in_insertion_order() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule(
            VirtualTime(100),
            Event::NodeFailed {
                node_id: "a".into(),
            },
        );
        scheduler.schedule(
            VirtualTime(100),
            Event::NodeFailed {
                node_id: "b".into(),
            },
        );
        scheduler.schedule(
            VirtualTime(100),
            Event::NodeFailed {
                node_id: "c".into(),
            },
        );

        let nodes: Vec<String> = std::iter::from_fn(|| scheduler.next_event())
            .map(|(_, e)| match e {
                Event::NodeFailed { node_id } => node_id,
                _ => "other".into(),
            })
            .collect();

        assert_eq!(nodes, vec!["a", "b", "c"]);
    }

    #[test]
    fn empty_scheduler_returns_none() {
        let mut scheduler = Scheduler::new();
        assert!(scheduler.is_empty());
        assert!(scheduler.next_event().is_none());
    }

    #[test]
    fn current_time_advances() {
        let mut scheduler = Scheduler::new();
        assert_eq!(scheduler.now().millis(), 0);

        scheduler.schedule(
            VirtualTime(500),
            Event::NodeRecovered {
                node_id: "x".into(),
            },
        );
        scheduler.next_event();

        assert_eq!(scheduler.now().millis(), 500);
    }
}
