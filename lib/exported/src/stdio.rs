use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use crate::server::*;

pub fn getchar() -> u8 {
  loop {
    let result = call(common::server::SERVER_TERMINAL, Message(
      0, 0, 0, 0
    )).unwrap();
    match result.0 {
      0 => microcall::thread_yield(),
      c => break c as u8,
    }
  }
}

pub fn getline() -> String {
  let mut v = Vec::new();
  loop {
    let c = getchar();
    if c == 0xd {
      break
    }
    v.push(c);
  }
  String::from_utf8(v).expect("getline failed!")
}


struct Writer;

static WRITER: Mutex<Writer> = Mutex::new(Writer);

impl core::fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    for c in s.chars() {
      microcall::putc(c);
    }
    Ok(())
  }
}

pub fn print_arg(args: core::fmt::Arguments) {
  use core::fmt::Write;
  let mut writer = WRITER.lock();
  writer.write_fmt(args).unwrap();
}
