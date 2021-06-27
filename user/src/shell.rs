#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate exported;

#[no_mangle]
fn _start() -> ! {
  exported::heap::init();
  println!("Welcome to rustpi shell!");
  loop {
    print!("SHELL> ");
    let cmd = exported::stdio::getline();
    println!();
    if cmd == "ls" {
      let mut root = fs::File::open("/").unwrap();
      let mut buf = [0u8; 128];
      root.read(&mut buf).unwrap();
      let dir = core::str::from_utf8(&buf).unwrap();
      println!("{}", dir);
    } else {
      exported::pm::exec(cmd.as_str());
    }
  }
}
