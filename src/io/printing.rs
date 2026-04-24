use core::{cell::SyncUnsafeCell, fmt};

use crate::{
    sync::nonpoison::Mutex,
    sys::{
        display::{
            sceDisplaySetFrameBuf, sceDisplaySetMode, DisplayMode, DisplayUpdateSync, PixelFormat,
        },
        ge::sceGeEdramGetAddr,
    },
};

/// Raw MSX font.
///
/// This is an 8bit x 256 black and white image.
const MSX_FONT: [u8; 2048] = *include_bytes!("msxfont.bin");

const BUFFER_WIDTH: usize = 512;
const DISPLAY_HEIGHT: usize = 272;
const DISPLAY_WIDTH: usize = 480;

static VRAM_BASE: SyncUnsafeCell<u32> = SyncUnsafeCell::new(0);

// TODO: Wrap this in some kind of a mutex.
// static CHARS: Mutex<SyncUnsafeCell<CharBuffer>> =
//     Mutex::new(SyncUnsafeCell::new(CharBuffer::new()));
static CHARS: SyncUnsafeCell<CharBuffer> = SyncUnsafeCell::new(CharBuffer::new());

// TODO: Wait for better const generics.
const ROWS: usize = MsxFont::ROWS;
const COLS: usize = MsxFont::COLS;

trait Font {
    const CHAR_WIDTH: usize;
    const CHAR_HEIGHT: usize;
    const ROWS: usize;
    const COLS: usize;

    fn put_char(x: usize, y: usize, color: u32, c: u8);
}

#[derive(Clone, Copy)]
struct MsxFont;

impl Font for MsxFont {
    const CHAR_HEIGHT: usize = 10;
    const CHAR_WIDTH: usize = 6;
    const COLS: usize = DISPLAY_WIDTH / MsxFont::CHAR_WIDTH;
    const ROWS: usize = DISPLAY_HEIGHT / MsxFont::CHAR_HEIGHT;

    fn put_char(x: usize, y: usize, color: u32, c: u8) {
        unsafe {
            let vram_base: *mut u32 =
                core::ptr::with_exposed_provenance_mut((*VRAM_BASE.get()) as usize);
            let mut ptr = vram_base.add(x + y * BUFFER_WIDTH);

            for i in 0..8 {
                for j in 0..8 {
                    if MSX_FONT[c as usize * 8 + i] & (0b1000_0000 >> j) != 0 {
                        *ptr = color;
                    }

                    ptr = ptr.offset(1);
                }

                ptr = ptr.add(BUFFER_WIDTH - 8);
            }
        }
    }
}

#[derive(Copy, Clone)]
struct Line {
    chars: [u8; COLS],
    len: usize,
}

impl Line {
    const fn new() -> Self {
        Self {
            chars: [0; COLS],
            len: 0,
        }
    }
}

struct CharBuffer {
    lines: [Line; ROWS],
    written: usize,
    advance_next: bool,
}

impl CharBuffer {
    const fn new() -> Self {
        Self {
            lines: [Line::new(); ROWS],
            written: 0,
            advance_next: false,
        }
    }

    fn advance(&mut self) {
        self.written += 1;
        if self.written >= ROWS {
            *self.current_line() = Line::new();
        }
    }

    fn current_line(&mut self) -> &mut Line {
        &mut self.lines[self.written % ROWS]
    }

    fn add(&mut self, c: u8) {
        if self.advance_next {
            self.advance_next = false;
            self.advance();
        }

        match c {
            b'\n' => self.advance_next = true,
            b'\t' => {
                self.add(b' ');
                self.add(b' ');
                self.add(b' ');
                self.add(b' ');
            },

            _ => {
                if self.current_line().len == COLS {
                    self.advance();
                }

                let line = self.current_line();
                line.chars[line.len] = c;
                line.len += 1;
            },
        }
    }

    fn lines(&self) -> LineIter<'_> {
        LineIter { buf: self, pos: 0 }
    }
}

impl fmt::Write for CharBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            match c as u32 {
                0..=255 => {
                    let chars = unsafe { &mut *CHARS.get() };
                    chars.add(c as u8)
                },
                _ => {
                    let chars = unsafe { &mut *CHARS.get() };
                    chars.add(0)
                },
            }
        }


        Ok(())
    }
}

struct LineIter<'a> {
    buf: &'a CharBuffer,
    pos: usize,
}

impl<'a> Iterator for LineIter<'a> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < core::cmp::min(self.buf.written + 1, ROWS) {
            let idx = if self.buf.written > ROWS {
                (self.buf.written + 1 + self.pos) % ROWS
            } else {
                self.pos
            };

            let line = self.buf.lines[idx];
            self.pos += 1;
            Some(line)
        } else {
            None
        }
    }
}

unsafe fn init() {
    unsafe {
        // The OR operation here specifies the address bypasses cache.
        let vram_base = VRAM_BASE.get();
        *vram_base = 0x4000_0000u32 | sceGeEdramGetAddr() as u32;

        // TODO: Change sys types to usize.
        let _res = sceDisplaySetMode(DisplayMode::Lcd, DISPLAY_WIDTH, DISPLAY_HEIGHT);
        let _res = sceDisplaySetFrameBuf(
            core::ptr::with_exposed_provenance_mut((*vram_base) as usize),
            BUFFER_WIDTH,
            PixelFormat::Psm8888,
            DisplayUpdateSync::NextVsync,
        );
    }
}

unsafe fn clear_screen(color: u32) {
    unsafe {
        let vram_base = *VRAM_BASE.get();
        let mut ptr: *mut u32 = core::ptr::with_exposed_provenance_mut(vram_base as usize);

        for _ in 0..(BUFFER_WIDTH * DISPLAY_HEIGHT) {
            *ptr = color;
            ptr = ptr.offset(1);
        }
    }
}

unsafe fn put_str<T: Font>(s: &[u8], x: usize, y: usize, color: u32) {
    if y > DISPLAY_HEIGHT {
        return;
    }

    for (i, c) in s.iter().enumerate() {
        if i >= (DISPLAY_WIDTH / T::CHAR_WIDTH) {
            break;
        }

        if *c as u32 <= 255 && *c != b'\0' {
            T::put_char(T::CHAR_WIDTH * i + x, y, color, *c);
        }
    }
}

/// Update the screen.
fn update() {
    unsafe {
        init();
        clear_screen(0);

        // let mut chars = CHARS.lock();
        let chars = &*CHARS.get();
        for (i, line) in chars.lines().enumerate() {
            put_str::<MsxFont>(&line.chars[0..line.len], 0, i * MsxFont::CHAR_HEIGHT, 0xFFFF_FFFF)
        }
    }
}

#[doc(hidden)]
pub fn _dprint(arguments: core::fmt::Arguments<'_>) {
    use fmt::Write;

    {
        let chars = unsafe { &mut *CHARS.get() };
        let _ = write!(chars, "{}", arguments);
    }

    update();
}

#[doc(hidden)]
pub fn _print(arguments: core::fmt::Arguments<'_>) {
    use alloc::vec::Vec;
    use io_core::io::Write;
    let mut buffer = Vec::with_capacity(256);

    let _ = write!(buffer, "{arguments}");

    let out = crate::sys::io::sceKernelStdout();
    let _res = unsafe { crate::sys::io::sceIoWrite(out, buffer.as_ptr(), buffer.len()) };
    if _res.is_err() {
        panic!("Failed to print to stdout");
    }
}
