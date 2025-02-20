#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate rpstdlib;


#[no_mangle]
fn _start(arg: *const u8) {
  let arg = rpstdlib::parse(arg);
  if arg.len() == 0 {
    println!("usage: rm FILE...");
    rpstdlib::exit();
  }
  let path = arg[0];
  match rpstdlib::fs::remove_file(path) {
    Ok(_) => {}
    Err(e) => {
      println!("{}", e);
    }
  }
  rpstdlib::exit();
}
