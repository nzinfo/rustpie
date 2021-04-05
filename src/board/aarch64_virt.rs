use core::ops::Range;

use crate::driver::gic::INT_TIMER;
use crate::lib::interrupt::InterruptController;

pub const BOARD_CORE_NUMBER: usize = 1;
#[allow(dead_code)]
pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0x8000_0000;
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x4000_0000..0x8000_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x4000_0000;
pub const BOARD_PHYSICAL_ENTRY: u64 = 0x40080000;


//pub const INT_VIRTIO_MMIO_0: Interrupt = 0x10 + 32;

pub fn init() {
  crate::driver::uart::init();
  crate::driver::common::virtio_blk::init();
  // crate::driver::INTERRUPT_CONTROLLER.enable(INT_VIRTIO_MMIO_0);
}

pub fn init_per_core() {
  use cortex_a::regs::*;
  DAIF.write(DAIF::I::Masked);
  crate::driver::INTERRUPT_CONTROLLER.init();
  crate::driver::INTERRUPT_CONTROLLER.enable(INT_TIMER);
  crate::driver::timer::init();
}

pub fn launch_other_cores() {
  let core_id = crate::core_id();
  for i in 0..BOARD_CORE_NUMBER {
    if i != core_id {
      crate::driver::psci::cpu_on(i as u64, BOARD_PHYSICAL_ENTRY, 0);
    }
  }
}
