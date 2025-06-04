pub mod gpio;
pub mod uart;

pub static DTB: &[u8] = include_bytes!("bcm2837b0/bcm2710-rpi-3-b-plus.dtb");
