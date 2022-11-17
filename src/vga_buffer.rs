use volatile::Volatile;
use core::fmt::Write;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
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
    LighRed,
    Pink,
    Yellow,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> Self {
        // First four bits define the foreground color, 
        // the next three bits define the background color.
        //
        // Example: 
        // foreground = Color(2), background = Color(3)
        // 
        // 2 as u8 = 00000010, 3 as u8 = 00000011
        //
        // 00000010 << 4  = 00100000
        // 00100000 | 00000011 = 00100011
        //
        // now our first four bits hold 3 (0011),
        // next three contain 2 (010),
        // last bit indicates if our char is blinking
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
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

// pub API
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

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(&s);
        Ok(())
    }
}

pub fn print_smth() {

    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LighRed, Color::DarkGray),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    };

    writer.write_string("test ");
    writer.write_string("LoŁ>l ");
    write!(writer, "{} + {} equals {}", 1, 1, 1+1).unwrap();
    writer.write_byte(b'\n');
    for _ in 0..BUFFER_WIDTH {
        writer.write_byte(b'T');
    }
    writer.write_string("Crazy shit.");
}