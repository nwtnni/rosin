#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        {
            $crate::print!($($arg)*);
            $crate::print!("\n");
        }
    };
}

#[rustfmt::skip]
macro_rules! make_log {
    ($name:ident, $header:expr, $dollar:tt) => {
        #[macro_export]
        macro_rules! $name {
            ($format:expr) => {
                $crate::$name!($format,)
            };
            ($format:expr, $dollar($arg:tt)*) => {
                {
                    let now = core::time::Duration::from($crate::time::Instant::now());
                    $crate::println!(
                        concat!("[", $header, "][{:>4}.{:06}][", core::module_path!(), "] ", $format),
                        now.as_secs(),
                        now.subsec_micros(),
                        $dollar($arg)*
                    )
                }
            };
        }
    };
}

make_log!(info, "INFO", $);
make_log!(warn, "WARN", $);
