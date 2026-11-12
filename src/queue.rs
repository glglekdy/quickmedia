//! 동시에 N개까지만 시작시키는 다운로드 대기열. 화면 없이 동작한다.

pub struct JobQueue<T: Clone + PartialEq> {
    limit: usize,
    start: Box<dyn FnMut(T)>,
    running: Vec<T>,
    pending: Vec<T>,
}

impl<T: Clone + PartialEq> JobQueue<T> {
    pub fn new(limit: usize, start: Box<dyn FnMut(T)>) -> Self {
        Self { limit: limit.max(1), start, running: Vec::new(), pending: Vec::new() }
    }

    pub fn running(&self) -> Vec<T> {
        self.running.clone()
    }

    pub fn pending(&self) -> Vec<T> {
        self.pending.clone()
    }

    pub fn add(&mut self, job: T) {
        self.pending.push(job);
        self.fill();
    }

    pub fn done(&mut self, job: &T) {
        if let Some(i) = self.running.iter().position(|j| j == job) {
            self.running.remove(i);
            self.fill();
        }
    }

    /// 줄여도 이미 돌던 작업은 끝까지 간다. 빈 자리가 한도 아래로 내려갈 때까지 새로 시작하지 않는다.
    pub fn set_limit(&mut self, n: usize) {
        self.limit = n.max(1);
        self.fill();
    }

    /// 아직 시작하지 않은 작업을 버린다(앱을 닫을 때).
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    fn fill(&mut self) {
        while !self.pending.is_empty() && self.running.len() < self.limit {
            let job = self.pending.remove(0);
            self.running.push(job.clone());
            (self.start)(job);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn make(limit: usize) -> (JobQueue<char>, Rc<RefCell<Vec<char>>>) {
        let started = Rc::new(RefCell::new(Vec::new()));
        let sink = started.clone();
        (JobQueue::new(limit, Box::new(move |j| sink.borrow_mut().push(j))), started)
    }

    #[test]
    fn starts_up_to_limit_in_order() {
        let (mut q, started) = make(2);
        for j in "abcd".chars() {
            q.add(j);
        }
        assert_eq!(*started.borrow(), ['a', 'b']);
        assert_eq!(q.pending(), ['c', 'd']);
    }

    #[test]
    fn done_starts_next() {
        let (mut q, started) = make(1);
        q.add('a');
        q.add('b');
        q.done(&'a');
        assert_eq!(*started.borrow(), ['a', 'b']);
        assert_eq!(q.running(), ['b']);
        assert!(q.pending().is_empty());
        q.done(&'z'); // 모르는 작업
        assert_eq!(q.running(), ['b']);
    }

    #[test]
    fn raise_limit_starts_immediately() {
        let (mut q, started) = make(1);
        for j in "abc".chars() {
            q.add(j);
        }
        q.set_limit(3);
        assert_eq!(*started.borrow(), ['a', 'b', 'c']);
    }

    #[test]
    fn lower_limit_keeps_running_jobs() {
        let (mut q, started) = make(3);
        for j in "abcd".chars() {
            q.add(j);
        }
        q.set_limit(1);
        assert_eq!(q.running(), ['a', 'b', 'c']);
        q.done(&'a');
        q.done(&'b');
        assert_eq!(*started.borrow(), ['a', 'b', 'c']); // 아직 한도(1)를 넘는다
        q.done(&'c');
        assert_eq!(*started.borrow(), ['a', 'b', 'c', 'd']);
    }

    #[test]
    fn zero_limit_is_treated_as_one() {
        let (mut q, started) = make(0);
        q.add('a');
        assert_eq!(*started.borrow(), ['a']);
    }

    #[test]
    fn clear_pending_drops_waiting_jobs_only() {
        let (mut q, started) = make(1);
        q.add('a');
        q.add('b');
        q.clear_pending();
        q.done(&'a');
        assert_eq!(*started.borrow(), ['a']);
        assert!(q.running().is_empty());
    }
}
