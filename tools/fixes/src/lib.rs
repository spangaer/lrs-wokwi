#![no_std]

#[macro_export]
macro_rules! println {
    ($no_format:expr) => {
        print!("{}\r\n",$no_format)
    };
    ($($arg:tt)*) => {
        print!("{}\r\n", format_args!($($arg)*))
    };
}
