use std::{cmp::Reverse, collections::BTreeSet};

pub(super) struct Queue<T, P>(BTreeSet<(P, Reverse<T>)>);

impl<T, P> Queue<T, P>
where
    T: Eq + Ord + Copy,
    P: Eq + Ord + Copy,
{
    pub(super) fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub(super) fn push(&mut self, element: T, priority: P) {
        self.0.insert((priority, Reverse(element)));
    }

    pub(super) fn remove(&mut self, element: T, priority: P) {
        self.0.remove(&(priority, Reverse(element)));
    }

    pub(super) fn pop(&mut self) -> Option<T> {
        self.0.pop_last().map(|(_, Reverse(element))| element)
    }
}
