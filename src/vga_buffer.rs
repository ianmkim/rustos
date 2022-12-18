use volatile::Volatile;
use core::fmt::Write;
use spin::Mutex;
use lazy_static::lazy_static;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;


#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

/*
 * the VGA text buffer:
 * 0-7   (8bits) -> ASCII code (actually its really code page 437)
 * 8-11  (4bits) -> foreground color
 * 12-14 (4bits) -> background color
 * 15    (1bit)  -> blink
 */
impl ColorCode{
    fn new(foreground: Color, background: Color) -> ColorCode {
        // background color first then shift 4 to the left
        // then add the foreground color as u8
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}


#[repr(transparent)]
struct Buffer {
    // chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    // this is to prevent the compiler from aggressive optimization. Since
    // we never read from the buffer and only write to it, the compiler might
    // eventually decide to get rid of it
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/*
 * the &'static explicitly defines the lifetime of the Buffer type stored
 * in writer.buffer. The static lifecycle will ensure that the buffer var
 * lasts for the entire runtime of the program
 */
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}


impl Writer {
    pub fn write_byte(&mut self, byte: u8){
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self){
        for row in 1..BUFFER_HEIGHT{
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row-1][col].write(character);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row:usize){
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH{
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str){
        // iterate over each character as a byte
        for byte in s.bytes() {
            // check whether the byte we want to write is within the
            // valid ascii range
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
}


impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}


lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position:0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}


#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}


#[macro_export]
macro_rules! println{
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}


#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}


/*
pub fn print_something(){
    // create a buffer at 0xb8000 as a raw pointer, then dereference it
    // then own it as a mutable reference
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as * mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello World!");

    write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
}
*/


#[test_case]
fn test_println_many(){
    for _ in 0..200 {
        println!("test_println_many_output")
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}
