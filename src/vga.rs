use core::fmt;
use core::fmt::Write as _;

pub mod buffer;
pub mod color;
pub mod screen;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    screen::WRITER.lock().write_fmt(args).unwrap();
}

#[cfg(test)]
mod tests {

    use crate::println;
    use super::buffer;
    use super::screen;

    #[test_case]
    fn println_no_panic() {
        println!("Hello, world!");
    }

    #[test_case]
    fn println_many() {
        for line in 0..200 {
            println!("Line {}", line);
        }
    }

    #[test_case]
    fn println_output() {
        let string = "Testing VGA buffer output";
        println!("{}", string);
        for (index, expected) in string.chars().enumerate() {
            let actual = screen::WRITER
                .lock()
                .buffer[(buffer::HEIGHT - 2, index)]
                .read();
            assert_eq!(
                char::from(actual.ascii),
                expected,
            );
        }
    }

}
