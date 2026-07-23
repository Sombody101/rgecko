use std::fmt::{Debug, Display, Formatter};

pub fn get_format_code_from_label(label: &str) -> Option<&'static str> {
    return __find_in_map(FORMAT_CODES, label);
}

// fn get_control_from_key(key: &str) -> Option<&'static str> {
//     return __find_in_map(CONTROL_CHARS, key);
// }

pub fn __find_in_map(map: &[(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    return map.iter().find(|&&(k, _)| k == key).map(|&(_, v)| v);
}

pub static FORMAT_CODES: &[(&'static str, &'static str)] = &[
    ("bold", ";1"),
    ("dim", ";2"),
    ("italic", ";3"),
    ("underlined", ";4"),
    ("blinking", ";5"),
    ("fblinking", ";6"),
    ("swap", ";7"),
    ("striked", ";9"),
    ("dunderlined", ";21"),
    ("overlined", ";53"),
];

pub static CONTROL_CHARS: &[(&'static str, &'static str)] = &[
    ("\\003", "\x03"),
    ("\\x1b", "\x1b"),
    ("\\n", "\n"),
    ("\\r", "\r"),
    ("\\t", "\t"),
    ("\\a", "\x07"),
    ("\\b", "\x08"),
    ("\\f", "\x0c"),
    ("\\v", "\x0b"),
];

pub trait AnsiConvert {
    fn sgr_str(self) -> &'static str;
}

impl AnsiConvert for u8 {
    #[inline]
    fn sgr_str(self) -> &'static str {
        match self {
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            9 => "9",
            21 => "21",
            53 => "53",
            _ => "",
        }
    }
}

pub struct AnsiCommand {
    pub style: Style,
    pub color: Option<Color>,
}

impl AnsiCommand {
    pub fn build_into(&self, layer: Layer, output: &mut String) {
        self.style.build_into(output);

        if let Some(color) = &self.color {
            output.push_str(layer.as_str());
            color.build_into(output);
        }
    }
}

//pub type Style = [u8; 10];

pub struct Style {
    flags: u16,
}

impl Style {
    pub const BOLD: u16 = 1 << 0;
    pub const DIM: u16 = 1 << 1;
    pub const ITALIC: u16 = 1 << 2;
    pub const UNDERLINE: u16 = 1 << 3;
    pub const BLINK: u16 = 1 << 4;
    pub const FAST_BLINK: u16 = 1 << 5;
    pub const INVERT: u16 = 1 << 6;
    pub const STRIKE: u16 = 1 << 7;
    pub const DUNDERLINE: u16 = 1 << 8;
    pub const OVERLINE: u16 = 1 << 9;

    pub const EMPTY: Self = Self { flags: 0 };

    #[inline]
    pub fn set_flag(&mut self, style: &str) -> Option<u16> {
        let flag = match style {
            "bold" => Self::BOLD,
            "dim" => Self::DIM,
            "italic" => Self::ITALIC,
            "underlined" => Self::UNDERLINE,
            "blinking" => Self::BLINK,
            "fblinking" => Self::FAST_BLINK,
            "swap" => Self::INVERT,
            "striked" => Self::STRIKE,
            "dunderlined" => Self::DUNDERLINE,
            "overlined" => Self::OVERLINE,
            _ => return None,
        };

        self.flags |= flag;
        Some(flag)
    }

    pub fn is_empty(&self) -> bool {
        self.flags == 0
    }

    #[inline]
    pub fn build_into(&self, output: &mut String) -> bool {
        if self.flags == 0 {
            return false;
        }

        let mut written = false;
        let mut check_bit = |flag: u16, code: &str| {
            if (self.flags & flag) != 0 {
                if written {
                    output.push(';');
                }
                output.push_str(code);
                written = true;
            }
        };

        check_bit(Self::BOLD, "1");
        check_bit(Self::DIM, "2");
        check_bit(Self::ITALIC, "3");
        check_bit(Self::UNDERLINE, "4");
        check_bit(Self::BLINK, "5");
        check_bit(Self::FAST_BLINK, "6");
        check_bit(Self::INVERT, "7");
        check_bit(Self::STRIKE, "9");
        check_bit(Self::DUNDERLINE, "21");
        check_bit(Self::OVERLINE, "53");

        written
    }
}

impl Default for Style {
    fn default() -> Self {
        Style { flags: 0 }
    }
}

impl Debug for Style {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.flags == 0 {
            return f.write_str("empty");
        }

        let mut written = false;

        let mut check_bit = |flag: u16, name: &str| -> std::fmt::Result {
            if (self.flags & flag) != 0 {
                if written {
                    f.write_str(" | ")?;
                }
                f.write_str(name)?;
                written = true;
            }
            Ok(())
        };

        check_bit(Self::BOLD, "1")?;
        check_bit(Self::DIM, "2")?;
        check_bit(Self::ITALIC, "3")?;
        check_bit(Self::UNDERLINE, "4")?;
        check_bit(Self::BLINK, "5")?;
        check_bit(Self::FAST_BLINK, "6")?;
        check_bit(Self::INVERT, "7")?;
        check_bit(Self::STRIKE, "9")?;
        check_bit(Self::DUNDERLINE, "21")?;
        check_bit(Self::OVERLINE, "53")
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    fn build_into(&self, output: &mut String) {
        output.push_str(";2;");
        push_u8(output, self.r);
        output.push(';');
        push_u8(output, self.g);
        output.push(';');
        push_u8(output, self.b);
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.r, self.g, self.b)
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Layer {
    Foreground = 38,
    Background = 48,
}

impl Layer {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Foreground => "38",
            Self::Background => "48",
        }
    }
}

fn push_u8(output: &mut String, byte: u8) {
    let mut buf = itoa::Buffer::new();
    let formatted = buf.format(byte);
    output.push_str(formatted);
}
