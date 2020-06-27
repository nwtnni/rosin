#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum T {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    Gray = 0x7,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Fore {
    pub bright: bool,
    pub color: T,
}

impl From<Fore> for u8 {
    fn from(fore: Fore) -> Self {
        (fore.bright as u8) << 4 | (fore.color as u8)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Back {
    pub blink: bool,
    pub color: T,
}

impl From<Back> for u8 {
    fn from(back: Back) -> Self {
        (back.blink as u8) << 4 | (back.color as u8)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Code(u8);

impl Code {
    pub fn new(fore: Fore, back: Back) -> Self {
        Code(u8::from(back) << 4 | u8::from(fore))
    }
}
