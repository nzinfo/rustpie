#![no_std]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![feature(lang_items)]
#![feature(box_syntax)]

extern crate alloc;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::print_arg(format_args_nl!($($arg)*));
    })
}

pub mod print;
pub mod thread;
pub mod mm;
pub mod redoxcall;
pub mod message;
pub mod loader;
pub mod fs;