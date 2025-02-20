#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(lang_items)]
#![feature(allocator_api)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate static_assertions;

use arch::ContextFrame;
use lib::interrupt::InterruptController;
use lib::traits::*;
use mm::page_table::PageTableEntryAttrTrait;
use mm::page_table::PageTableTrait;

#[macro_use]
mod misc;

cfg_if::cfg_if! {
  if #[cfg(target_arch = "aarch64")] {
    #[path = "arch/aarch64/mod.rs"]
    mod arch;

    #[cfg(not(feature = "tx2"))]
    #[path = "board/aarch64_virt.rs"]
    mod board;

    #[cfg(feature = "tx2")]
    #[path = "board/aarch64_tx2.rs"]
    mod board;

    #[path = "driver/aarch64/mod.rs"]

    mod driver;
    // Note: size of ContextFrame needs to be synced with `arch/*/exception.S`
    assert_eq_size!([u8; 0x110], ContextFrame);
  } else if #[cfg(target_arch = "riscv64")] {
    #[path = "arch/riscv64/mod.rs"]
    mod arch;

    #[cfg(feature = "k210")]
    #[path = "board/riscv64_k210.rs"]
    mod board;

    #[cfg(not(feature = "k210"))]
    #[path = "board/riscv64_virt.rs"]
    mod board;

    #[path = "driver/riscv64/mod.rs"]
    mod driver;
    assert_eq_size!([u8; 0x110], ContextFrame);
  } else {
    compile_error!("unsupported target_arch");
  }
}


mod lib;
mod mm;
mod panic;
mod util;
mod logger;
mod syscall;

#[macro_use]
mod macros {
  #[repr(C)] // guarantee 'bytes' comes after '_align'
  pub struct AlignedAs<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes,
  }

  macro_rules! include_bytes_align_as {
  ($align_ty:ty, $path:literal) => {
    {  // const block expression to encapsulate the static
      use $crate::macros::AlignedAs;

      // this assignment is made possible by CoerceUnsized
      static ALIGNED: &AlignedAs::<$align_ty, [u8]> = &AlignedAs {
        _align: [],
        bytes: *include_bytes!($path),
      };

      &ALIGNED.bytes
    }
  };
}
}

#[repr(align(4096))]
struct AlignPage;

#[allow(dead_code)]
fn test_create_as() {
  let mut results = vec![];
  for _ in 0..1000 {
    let icntr = lib::timer::current_cycle();
    let a = lib::address_space::address_space_alloc().unwrap();
    let icntr2 = lib::timer::current_cycle();
    lib::address_space::address_space_destroy(a);
    results.push(icntr2 - icntr);
  }
  let mut sum = 0;
  for result in results {
    sum += result;
  }
  info!("[[TEST]] test_create_as {}/1000", sum);
}

#[allow(dead_code)]
fn test_create_thread() {
  let a = lib::address_space::address_space_alloc().unwrap();
  let mut results = vec![];
  for _ in 0..1000 {
    let icntr = lib::timer::current_cycle();
    let t = lib::thread::new_user(
      0x40000,
      rpabi::CONFIG_USER_STACK_TOP,
      0,
      a.clone(),
      None,
    );
    let icntr2 = lib::timer::current_cycle();
    lib::thread::thread_destroy(t);
    lib::address_space::address_space_destroy(a.clone());
    results.push(icntr2 - icntr);
  }
  let mut sum = 0;
  for result in results {
    sum += result;
  }
  info!("[[TEST]] test_create_thread {}/1000", sum);
}

#[no_mangle]
pub unsafe fn main(core_id: arch::CoreId) -> ! {
  crate::arch::Arch::exception_init();
  if core_id == 0 {
    board::init();
    mm::heap::init();
    let _ = logger::init();
    info!("heap init ok");
    mm::page_pool::init();
    info!("page pool init ok");

    board::launch_other_cores();
    info!("launched other cores");
  }
  board::init_per_core();
  info!("init core {}", core_id);

  if core_id == 0 {
    // {
    //   test_create_thread();
    //   test_create_as();
    //   loop {}
    // }

    #[cfg(target_arch = "aarch64")]
      #[cfg(not(feature = "user_release"))]
      let bin = include_bytes_align_as!(AlignPage, "../trusted/target/aarch64/debug/trusted");

    #[cfg(target_arch = "aarch64")]
      #[cfg(feature = "user_release")]
      let bin = include_bytes_align_as!(AlignPage, "../trusted/target/aarch64/release/trusted");

    #[cfg(target_arch = "riscv64")]
      #[cfg(not(feature = "user_release"))]
      let bin = include_bytes_align_as!(AlignPage, "../trusted/target/riscv64/debug/trusted");

    #[cfg(target_arch = "riscv64")]
      #[cfg(feature = "user_release")]
      let bin = include_bytes_align_as!(AlignPage, "../trusted/target/riscv64/release/trusted");

    info!("embedded trusted {:x}", bin.as_ptr() as usize);
    let (a, entry) = lib::address_space::load_image(bin);
    info!("load_image ok");

    let page_table = a.page_table();
    let stack_frame = mm::page_pool::page_alloc().expect("failed to allocate trusted stack");
    page_table.insert_page(rpabi::CONFIG_USER_STACK_TOP - arch::PAGE_SIZE,
                           mm::Frame::from(stack_frame),
                           mm::page_table::EntryAttribute::user_default()).unwrap();

    #[cfg(feature = "k210")]
      {
        let dma_frame = mm::page_pool::page_alloc().expect("failed to allocate trusted dma frame");
        let dma_frame_no_cache = dma_frame.pa() - 0x40000000;
        info!("dma_frame {:016x}", dma_frame_no_cache);
        page_table.insert_page(0x8_0000_0000,
                               mm::Frame::Device(dma_frame_no_cache),
                               mm::page_table::EntryAttribute::user_device()).unwrap();
        core::mem::forget(dma_frame);
      }

    info!("user stack ok");
    let t = crate::lib::thread::new_user(
      entry,
      rpabi::CONFIG_USER_STACK_TOP,
      0,
      a.clone(),
      None,
    );
    lib::thread::thread_wake(&t);

    for device in board::devices() {
      for uf in device.to_user_frames().iter() {
        a.page_table().insert_page(
          0x8_0000_0000 + uf.pa(),
          uf.clone(),
          mm::page_table::EntryAttribute::user_device(),
        ).unwrap();
      }
      for i in device.interrupts.iter() {
        crate::driver::INTERRUPT_CONTROLLER.enable(*i);
      }
    }
    info!("device added to user space");
  }

  util::barrier();
  lib::cpu::cpu().schedule();

  extern {
    fn pop_context_first(ctx: usize, core_id: usize) -> !;
  }
  match lib::cpu::cpu().running_thread() {
    None => panic!("no running thread"),
    Some(t) => {
      let ctx = t.context();
      pop_context_first(&ctx as *const _ as usize, core_id);
    }
  }
}
