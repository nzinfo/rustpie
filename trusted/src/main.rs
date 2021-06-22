#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(const_btree_new)]

#[macro_use]
extern crate alloc;
extern crate rlibc;

#[macro_use]
extern crate libtrusted;

mod blk;
mod fs;
mod root;

#[no_mangle]
fn _start(arg: usize) -> ! {
  libtrusted::mm::page_fault_init();
  libtrusted::mm::heap_init();
  root::main(arg);
  microcall::thread_destroy(0);
  loop {};
}
