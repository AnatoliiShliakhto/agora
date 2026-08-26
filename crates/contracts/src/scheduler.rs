//! Logical-time scheduler.
//!
//! Wall-clock time never enters the domain. The runtime feeds `Tick` events with a monotonic
//! logical timestamp; the scheduler releases due obligations in deterministic order.

use std::collections::BTreeSet;

use agora_domain::ids::ContractId;
pub use agora_domain::time::LogicalTime;

/// Due-time index of contract obligations.
#[derive(Debug, Default)]
pub struct Scheduler {
    due: BTreeSet<(LogicalTime, ContractId)>,
}

impl Scheduler {
    /// Empty scheduler.
    #[must_use]
    pub const fn new() -> Self {
        Self { due: BTreeSet::new() }
    }

    /// Registers `contract` to be woken at `at`.
    pub fn schedule(&mut self, at: LogicalTime, contract: ContractId) {
        self.due.insert((at, contract));
    }

    /// Removes and returns every contract due at or before `now`, earliest first.
    pub fn drain_due(&mut self, now: LogicalTime) -> Vec<ContractId> {
        let mut ready = Vec::new();
        while let Some(&(at, contract)) = self.due.first() {
            if at > now {
                break;
            }
            self.due.remove(&(at, contract));
            ready.push(contract);
        }
        ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drains_in_time_then_id_order() {
        let mut s = Scheduler::new();
        let at = LogicalTime::from_nanos;
        s.schedule(at(20), ContractId::new(1));
        s.schedule(at(10), ContractId::new(9));
        s.schedule(at(10), ContractId::new(2));
        s.schedule(at(30), ContractId::new(3));
        let ready: Vec<_> = s.drain_due(at(20)).into_iter().map(ContractId::raw).collect();
        assert_eq!(ready, vec![2, 9, 1]);
        assert_eq!(s.drain_due(at(25)), vec![]);
    }
}
