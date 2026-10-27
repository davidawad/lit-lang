//! Pending moves (SPEC §9.3): a request an `extend` grant of a deadline
//! reads (`on M()`: a stipulation, a motion to extend) that the matter
//! filed for an instance no order has yet answered. The instance carries
//! it, filed and awaiting order, until the order extending it or the
//! denial the grant names (`unless D()`) arrives; the request is then
//! history again.
use super::Cx;
use crate::case::graph::{display, pack_of};
use crate::case::Pending;
use crate::temporal::condition::matches;

impl Cx<'_, '_> {
    /// At `as_of`: the pending move of every instance no event ended.
    pub(super) fn pending_at_as_of(&mut self) {
        for i in 0..self.obligations.len() {
            let pending = self.pending(i);
            self.obligations[i].pending = pending;
        }
    }

    /// The earliest request instance `i` awaits an order on: after its
    /// trigger and its last extending order, matching the motion of a
    /// grant of the version governing it at `as_of`, and not denied
    /// since.
    fn pending(&self, i: usize) -> Option<Pending> {
        let o = &self.obligations[i];
        if o.closed_by.is_some() {
            return None;
        }
        let (w, d) = self.governing(&o.deadline, self.sorted.len(), true)?;
        let pack = pack_of(&w);
        let after = o.answered.unwrap_or(o.at);
        let n = self.sorted.len();
        self.extend_grants(&w, d, &self.case.as_of)
            .into_iter()
            .filter_map(|g| g.motion)
            .filter_map(|m| {
                let j = (after + 1..n).find(|&j| matches(&m.event, self.view(pack, j)))?;
                let denied = m
                    .unless
                    .as_ref()
                    .is_some_and(|u| (j + 1..n).any(|k| matches(u, self.view(pack, k))));
                (!denied).then_some(j)
            })
            .min()
            .map(|j| {
                let (event, e) = self.sorted[j];
                Pending {
                    event,
                    request: display(&e.action),
                    date: e.date.clone(),
                }
            })
    }
}
