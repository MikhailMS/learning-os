use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile; // Need to use volatile so that compiler will not optimise away writes to Buffer.chars

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // To ensure enum is C-like: each enum variant is stored as u8 value
pub enum Colour {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // To ensure memory layout is exactly same as for u8
pub struct ColourCode(u8);
/*
 * | Bits  |       Value       |
 * | 0-7   | ASCII code point  |
 * | 8-11  | Foreground colour |
 * | 12-14 | Background colour |
 * | 15    | Blink             |
 */
impl ColourCode {
    pub fn new(foreground: Colour, background: Colour) -> ColourCode {
        ColourCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // To ensure field ordering (Rust ordering in default structs is undefined)
struct ScreenChar {
    ascii_char:  u8,
    colour_code: ColourCode
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH:  usize = 80;

#[repr(transparent)] // To ensure memory layout is exactly same as for its single field
pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT], // chars represents the VGA output memory area
}

pub struct Writer {
    pub column_pos:  usize,
    pub colour_code: ColourCode,
    pub buffer:      &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte  => {
                if self.column_pos >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_pos;

                let colour_code = self.colour_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_char: byte,
                    colour_code,
                });

                self.column_pos += 1;
            }
        }
    }

    pub fn write_byte_at(&mut self, byte: u8, row: usize, col: usize) {
        match byte {
            // printable ASCII byte or newline
            0x20..=0x7e => {
                if (row < BUFFER_HEIGHT) && (col < BUFFER_WIDTH) {
                    let colour_code = self.colour_code;
                    self.buffer.chars[row][col].write(ScreenChar {
                        ascii_char: byte,
                        colour_code,
                    });
                }
            },
            // not part of printable ASCII range
            _ => (),
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_pos = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_char:  b' ',
            colour_code: self.colour_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Only to be used by macros!
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap(); // Assumably alright to unwrap, because we always return Ok(()) from write_str()
}


lazy_static! {
    // Create static WRITER so it could be imported by other modules
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_pos:  0,
        colour_code: ColourCode::new(Colour::Yellow, Colour::Black),
        /*
         * () gives a mutable raw pointer, casted as *mut Buffer ([[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT], where ScreenChar is (u8, u8))
         * then *() dereference the pointer (unsafe operation)
         * and then &mut turns everything into mutable reference
         *
         * I think what happens here is that we (very roughly) substitute raw VGA buffer (memory space) with our custom Buffer and it works
         * as expected because memory layout of custom Buffer is identical to one of raw VGA buffer.
         * However, our custom Buffer has guarantees that it won't go over the limits because it is hardcoded with VGA supported size (25 * 80)
         *      ^ 
         *   0  |
         *      |
         *      |
         *      |
         *      |
         *      |
         * r    |
         * o    |
         * w 24 |--------------------> 
         *      0 col               80
         *
         *      direction: bottom - up
         *
         *  To figure out if our VGA supports colour or monochrome
         *  // Step 1. Get pointer to 0x410 - memory location where this data is stored
         *  let video = unsafe { &mut *(0x410 as *mut u16) };
         *  // Step 2. Value at pointer & 0x30 (logical AND): 0x00 - none, 0x20 - colour, 0x30 - mono
         *  let colour_mode = *video & 0x30;
         *  // Step 3. Depending on VGA support mode, we need to adjust address of the VGA buffer: 0xb8000 - colour, 0xb0000 - monochrome
         *  
         *  // 0xA0000 - memory address for raw frame buffer
         */
        buffer:      unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}
