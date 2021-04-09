use crate::fs::{Disk, BLOCK_SIZE};
use crate::syscall::*;
use crate::virtio_blk::{BLK_SERVER, read_msg};
use crate::config::PAGE_SIZE;
use crate::itc::ItcMessage;
use crate::mem::valloc;
use rlibc::memcpy;

pub struct VirtioClient {
    server_tid: u16,
}

impl VirtioClient {
    pub fn new() -> VirtioClient {
        loop {
            // Wait for block server
            if BLK_SERVER.get().is_some() {
                break;
            }
        }
        VirtioClient {
            server_tid: *BLK_SERVER.get().unwrap(),
        }
    }
}

impl Disk for VirtioClient {
    fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize> {
        println!("block {:016x} buffer {:016x} len {}", block, buffer.as_mut_ptr() as usize, buffer.len());
        assert_eq!(buffer.len() % BLOCK_SIZE as usize, 0);
        assert_eq!(buffer.len(), PAGE_SIZE);
        let tmp =valloc(1);
        let r = read_msg((block as usize) * 8, 8, unsafe { core::slice::from_raw_parts_mut(tmp, PAGE_SIZE) }).send_to(self.server_tid);
        assert_eq!(r, 0);
        let msg = ItcMessage::receive();
        // assert_eq!(msg.0, self.server_tid);
        assert_eq!(msg.1.a, 0);
        unsafe { memcpy(buffer.as_mut_ptr(), tmp, PAGE_SIZE); }
        Ok(buffer.len())
    }

    fn write_at(&mut self, block: u64, buffer: &[u8]) -> Result<usize> {
        unimplemented!()
    }

    fn size(&mut self) -> Result<u64> {
        // TODO: obtain real size
        Ok(536870912) // 512MB
    }
}
