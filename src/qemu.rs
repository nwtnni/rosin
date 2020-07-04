use x86_64::instructions::port;

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Exit {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit(code: Exit) {
    unsafe {
        let mut port = port::PortWriteOnly::new(0xF4);
        port.write(code as u32);
    }
}
