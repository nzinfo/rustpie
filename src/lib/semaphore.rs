use alloc::collections::VecDeque;

use spin::Mutex;

use crate::lib::thread::Thread;

struct Inner {
  value: usize,
  queue: VecDeque<Thread>,
}

pub struct Semaphore {
  inner: Mutex<Inner>,
}

pub enum SemaphoreWaitResult {
  Acquired,
  Enqueued,
}

impl Semaphore {
  pub fn new() -> Self {
    Semaphore {
      inner: Mutex::new(Inner {
        value: 0,
        queue: VecDeque::new(),
      })
    }
  }

  pub fn wait(&self, t: Thread) -> SemaphoreWaitResult {
    let mut inner = self.inner.lock();
    if inner.value == 0 {
      t.sleep();
      inner.queue.push_back(t);
      SemaphoreWaitResult::Enqueued
    } else {
      inner.value -= 1;
      SemaphoreWaitResult::Acquired
    }
  }

  pub fn signal(&self) {
    let mut inner = self.inner.lock();
    if inner.value != 0 {
      inner.value += 1;
    } else {
      if let Some(t) = inner.queue.pop_front() {
        t.wake();
        crate::current_cpu().schedule();
      }
    }
  }

  pub fn try_signal(&self) -> Option<Thread> {
    let mut inner = self.inner.lock();
    inner.value = 0;
    if let Some(t) = inner.queue.pop_front() {
      crate::current_cpu().schedule();
      Some(t)
    } else {
      None
    }
  }
}

