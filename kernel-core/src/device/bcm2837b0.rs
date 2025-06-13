pub mod clock;
pub mod gpio;
pub mod ic;
pub mod mini;
pub mod uart;

pub static DTB: device_tree::Blob =
    device_tree::Blob::new(include_bytes!("bcm2837b0/bcm2710-rpi-3-b-plus.dtb"));
