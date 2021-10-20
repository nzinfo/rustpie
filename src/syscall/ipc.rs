use super::{Result, SyscallOutRegisters::*};
use crate::lib::thread::{thread_sleep, thread_wake, Tid};
use common::syscall::error::*;
use crate::lib::cpu::cpu;
use crate::lib::scheduler::scheduler;
use crate::lib::traits::ContextFrameTrait;
use crate::lib::thread::Status as ThreadStatus;

#[inline(never)]
pub fn itc_receive() -> Result {
  let t = super::current_thread()?;
  thread_sleep(&t, ThreadStatus::WaitForRequest);
  Ok(Unit)
}

#[inline(never)]
pub fn itc_send(tid: Tid, a: usize, b: usize, c: usize, d: usize) -> Result {
  let current = super::current_thread()?;
  let target = crate::lib::thread::thread_lookup(tid).ok_or_else(|| ERROR_INVARG)?;
  if target.wait_for_reply(|| {
    target.map_with_context(|ctx| {
      ctx.set_syscall_result(&Result::Ok(Pentad(current.tid() as usize, a, b, c, d)));
    });
  }, ThreadStatus::Runnable) {
    Ok(Unit)
  } else {
    Err(ERROR_DENIED)
  }
}

#[inline(never)]
pub fn itc_call(tid: Tid, a: usize, b: usize, c: usize, d: usize) -> Result {
  let current = super::current_thread()?;
  let target = crate::lib::thread::thread_lookup(tid).ok_or_else(|| ERROR_INVARG)?;
  if target.wait_for_request(|| {
    target.map_with_context(|ctx| {
      ctx.set_syscall_result(&Result::Ok(Pentad(current.tid() as usize, a, b, c, d)));
    });
    thread_sleep(&current, crate::lib::thread::Status::WaitForReply);
  }, ThreadStatus::Runnable) {
    Ok(Unit)
  } else {
    Err(ERROR_HOLD_ON)
  }
}
