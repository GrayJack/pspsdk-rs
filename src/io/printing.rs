use core::{
    cell::UnsafeCell,
    fmt,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    sync::nonpoison::ReentrantLock,
    sys::{
        display::{
            sceDisplaySetFrameBuf, sceDisplaySetMode, DisplayMode, DisplayUpdateSync, PixelFormat,
        },
        ge::sceGeEdramGetAddr,
        sync::SpinMutex,
    },
};

/// Raw MSX font.
///
/// This is an 8bit x 256 black and white image.
const MSX_FONT: [u8; 2048] = *include_bytes!("msxfont.bin");

const BUFFER_WIDTH: usize = 512;
const DISPLAY_HEIGHT: usize = 272;
const DISPLAY_WIDTH: usize = 480;

static VRAM_BASE: AtomicPtr<u32> = AtomicPtr::new(core::ptr::null_mut());

static CHARS: ReentrantLock<UnsafeCell<CharBuffer>, SpinMutex> =
    ReentrantLock::new_with(UnsafeCell::new(CharBuffer::new()), SpinMutex::new());

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
            let vram_base: *mut u32 = VRAM_BASE.load(Ordering::Relaxed);
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
                0..=255 => self.add(c as u8),
                _ => self.add(0),
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
        VRAM_BASE.store(
            (0x4000_0000_usize | sceGeEdramGetAddr() as usize) as *mut _,
            Ordering::Release,
        );
        let vram_base = VRAM_BASE.load(Ordering::Acquire);

        // TODO: Change sys types to usize.
        let _res = sceDisplaySetMode(DisplayMode::Lcd, DISPLAY_WIDTH, DISPLAY_HEIGHT);
        let _res = sceDisplaySetFrameBuf(
            vram_base.cast(),
            BUFFER_WIDTH,
            PixelFormat::Psm8888,
            DisplayUpdateSync::NextVsync,
        );
    }
}

unsafe fn clear_screen(color: u32) {
    unsafe {
        let vram_base = VRAM_BASE.load(Ordering::Relaxed);
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

        let chars = CHARS.lock();
        let chars = &mut *chars.get();
        for (i, line) in chars.lines().enumerate() {
            put_str::<MsxFont>(&line.chars[0..line.len], 0, i * MsxFont::CHAR_HEIGHT, 0xFFFF_FFFF)
        }
    }
}


fn print_to<T: crate::io::Write>(args: fmt::Arguments<'_>, global_s: fn() -> T, label: &str) {
    if let Err(e) = global_s().write_fmt(args) {
        panic!("failed printing to {label}: {e}");
    }
}

#[doc(hidden)]
pub fn _dprint(arguments: core::fmt::Arguments<'_>) {
    use fmt::Write;

    {
        let chars = CHARS.lock();
        let chars = unsafe { &mut *chars.get() };
        if let Err(e) = chars.write_fmt(arguments) {
            panic!("failed printing to debug screen: {e}");
        }
    }

    update();
}

#[doc(hidden)]
pub fn _print(arguments: core::fmt::Arguments<'_>) {
    use crate::io;

    print_to(arguments, io::stdout, "stdout")
}

#[doc(hidden)]
pub fn _eprint(arguments: core::fmt::Arguments<'_>) {
    use crate::io;

    print_to(arguments, io::stderr, "stderr")
}

/// Used by impl Termination for Result to print error after `main` or a test
/// has returned. Should avoid panicking, although we can't help it if one of
/// the Display impls inside args decides to.
pub(crate) fn attempt_print_to_stderr(args: fmt::Arguments<'_>) {
    use crate::io::{self, Write};
    // Ignore error if the write fails, for example because stderr is already
    // closed. There is not much point panicking at this point.
    let _ = io::stderr().write_fmt(args);
}
