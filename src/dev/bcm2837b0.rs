use crate::dev;

pub mod clock;
pub mod gpio;
pub mod ic;
pub mod uart;

pub static DTB: dev::tree::Blob =
    dev::tree::Blob::new(include_bytes!("bcm2837b0/bcm2710-rpi-3-b-plus.dtb"));
