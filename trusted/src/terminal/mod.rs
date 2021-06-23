use register::*;
use register::mmio::*;
use alloc::vec::Vec;
use spin::{Mutex, Once, Spin};
use alloc::collections::VecDeque;
use microcall::get_tid;

#[cfg(target_arch = "aarch64")]
const PL011_MMIO_BASE: usize = 0x8_0000_0000 + 0x900_0000;

register_structs! {
  #[allow(non_snake_case)]
  Pl011MmioBlock {
    (0x000 => Data: ReadWrite<u32>),
    (0x004 => RecvStatusErrClr: ReadWrite<u32>),
    (0x008 => _reserved_0),
    (0x018 => Flag: ReadOnly<u32>),
    (0x01c => _reserved_1),
    (0x020 => IrDALowPower: ReadWrite<u32>),
    (0x024 => IntBaudRate: ReadWrite<u32>),
    (0x028 => FracBaudRate: ReadWrite<u32>),
    (0x02c => LineControl: ReadWrite<u32>),
    (0x030 => Control: ReadWrite<u32>),
    (0x034 => IntFIFOLevel: ReadWrite<u32>),
    (0x038 => IntMaskSetClr: ReadWrite<u32>),
    (0x03c => RawIntStatus: ReadOnly<u32>),
    (0x040 => MaskedIntStatus: ReadOnly<u32>),
    (0x044 => IntClear: WriteOnly<u32>),
    (0x048 => DmaControl: ReadWrite<u32>),
    (0x04c => _reserved_2),
    (0x1000 => @END),
  }
}

struct Pl011Mmio {
  base_addr: usize,
}

impl core::ops::Deref for Pl011Mmio {
  type Target = Pl011MmioBlock;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr() }
  }
}

impl Pl011Mmio {
  const fn new(base_addr: usize) -> Self { Pl011Mmio { base_addr } }
  fn ptr(&self) -> *const Pl011MmioBlock { self.base_addr as *const _ }
}

static PL011_MMIO: Pl011Mmio = Pl011Mmio::new(PL011_MMIO_BASE);

fn wait_for_irq() {
  #[cfg(target_arch = "aarch64")]
    microcall::event_handler(0, 0, 0, 0x1 + 32);
}

fn irq() {
  let pl011 = &PL011_MMIO;
  let status = pl011.RawIntStatus.get();
  if status != 0 {
    pl011.IntClear.set(status);
  }
  loop {
    if pl011.Flag.get() & 0b1_0000 != 0 { // RXFE
      break;
    } else {
      let c = pl011.Data.get() & 0xff;
      let mut buf = buffer().lock();
      buf.push_back(c as u8);
    }
  }
}

pub fn input_server() {
  let pl011 = &PL011_MMIO;
  pl011.Control.set(0x301);
  pl011.IntClear.set(0b111_1111_1111);
  pl011.IntMaskSetClr.set(0b1_0000);
  loop {
    wait_for_irq();
    irq();
  }
}

static BUFFER: Once<Mutex<VecDeque<u8>>> = Once::new();

fn buffer() -> &'static Mutex<VecDeque<u8>> {
  match BUFFER.get() {
    None => { BUFFER.call_once(|| Mutex::new(VecDeque::new())); buffer() }
    Some(x) => {x}
  }
}

pub fn server() {
  println!("[TERMINAL] server started t{}",  get_tid());
  microcall::server_register(common::server::SERVER_TERMINAL).unwrap();

  loop {
    let (_client_tid, _msg) = libtrusted::message::Message::receive();
    let mut msg = libtrusted::message::Message::default();
    let mut buf = buffer().lock();
    match buf.pop_front() {
      None => { msg.a = 0 }
      Some(c) => { msg.a = c as usize }
    }
    msg.reply();
  }
}