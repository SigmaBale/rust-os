use uart_16550::SerialPort;
use spin::Mutex;

lazy_static::lazy_static! {
    // We are using uart_16550 crate for writing and instantiating our SerialPort.
    // Universal asynchronous receiver-transmitter is a device for serial communication,
    // and uart_16550 is universal device (means we got it on our chipset).
    // Also in Cargo.toml for bootimage crate we specify that serial output should be stdout.
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // Serial port - COM1. Port number = 0x3F8
        // Function is unsafe because it expects address of first serial port as input.
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

// Macros for printing to serial port, SerialPort already has implementation of Write trait.
// All inline assembly is inisde uart_16550 crate for writing and reading to the ports (in, out - assembly instructions).
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}