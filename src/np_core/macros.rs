#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        println!("{:?}", ($($arg)*));
    };
}