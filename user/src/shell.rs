#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate rpstdlib;


#[no_mangle]
fn _start() -> ! {
  rpstdlib::heap::init();
  println!("Welcome to rustpi shell!");
  let auto_command = vec![
    // "ls",
    // "mkdir thisadir",
    // "rd thisadir",
    // "touch thisafile",
    // "rm thisafile",
    // "write thisbfile hahahah",
    // "rm thisbfile",
  ];

  for cmd in auto_command {
    println!("AUTO> {}", cmd);
    match rpstdlib::pm::exec(cmd) {
      Ok(pid) => {
        rpstdlib::pm::wait(pid);
      }
      Err(e) => {
        println!("exec failed: {}", e);
      }
    }
  }

  loop {
    print!("SHELL> ");
    let cmd = rpstdlib::stdio::getline();
    println!();
    if cmd.trim().is_empty() {
      continue;
    }
    match rpstdlib::pm::exec(cmd.as_str()) {
      Ok(pid) => {
        rpstdlib::pm::wait(pid);
      }
      Err(e) => {
        println!("exec failed: {}", e);
      }
    }
  }
}

