use volatile::Volatile;
use spin::Mutex;

lazy_static::lazy_static! {
    /// This is a VGA buffer.
    /// 
    /// `Writer` wrapped inside `Mutex`, this makes *WRITER* `Sync` (thread safe) 
    /// preventing data races and at the same time providing interior mutability.
    /// 
    /// #### WRITER is a global mutable thread-safe variable.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
// Color representation enum.
// Variant determinants go from 0 to 15.
enum Color {
    Black = 0,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    Pink,
    Yellow,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
// Newtype wrapper for our second byte that represents background/foreground
// color for our ascii character (first byte)
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> Self {
        // First four bits define the foreground color, 
        // the next three bits define the background color.
        //
        // Example: 
        // foreground = Color(2), background = Color(3)
        // 
        // 2 as u8 = 0000_0010, 3 as u8 = 0000_0011
        //
        // 0000_0010 << 4  = 0010_0000
        // 0010_0000 | 0000_0011 = 0010_0011
        //
        // now our first (lowest) four bits hold 3 (0011) = Cyan [background]
        // next three bits contain 2 (010) = Green [foreground]
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
// explicitly implying we want memory layout of C for this struct fields
// this way ascii_character is at offset 0, color_code is at offset 1(8 bits)
// so our layout matches the vga text format.
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// vga_buffer size
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
// using repr(transparent) to indicate Buffer is only a Newtype
struct Buffer {
    // chars is representation of vga_buffer in memory
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// [`Writer`] that can directly write to vga_buffer.
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    // if our column_position is greater or equal to vga_buffer width
    // or byte equals to new_line, we copy and write the values from the
    // current row to the preceding row. (check new_line method)
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line()
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: b,
                    color_code: self.color_code
                });

                self.column_position += 1;
            }
        }
    }

    // each row gets copied and rewritten to the preceding row
    // after that we clear the last row in vga_buffer and reset position
    fn new_line(&mut self) { 
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row-1][col].write(char);
            }            
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    // clears the row by filling it with blank bytes -> b' '
    // blank bytes will still have same color formatting defined in `Writer`,
    // meaning only background color will be visible.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for column in 0..BUFFER_WIDTH {
            self.buffer.chars[row][column].write(blank);
        }
    }
}

// Public API of our Writer, this is one of core functions that allows us to write
// bytes into our vga_buffer.
// Only ASCII bytes from 0x20 (hexadecimal) through 0x7e including new_line (\n) are valid.
// Invalid ASCII is printed as 0xfe. (check README.md for valid ASCII table)
impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
}

// Implementing Write trait so we can write and format into our buffer in various ways,
// this is a must for ergonomics and for ease of use (macros are very nice)
impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(&s);
        Ok(())
    }
}

// Macro juice oh god, I just copy pasted it all from stdlib and changed crate path
// Also _print function is changed so it writes to WRITER (global)
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Writes directly to vga_buffer aka WRITER (global buff)
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[test_case]
fn test_println_output() {
    use x86_64::instructions::interrupts;
    use core::fmt::Write;

    let output = "Check if this string is in the vga_buffer.";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", output).expect("writeln! failed");
        for (i, char) in output.chars().enumerate() {
            let buffer_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(buffer_char.ascii_character as char, char);
        }
    });
}