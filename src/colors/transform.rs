use crate::argparse::parser::ColorMode;
use crate::colors::ansicodes::{CONTROL_CHARS, get_format_code_from_label};
use crate::colors::colorsheet::{COLORS, get_color_by_name};
use crate::logger::Logger;
use crate::v_log;

const RESET_COLOR: &str = "\x1b[0m";
const READABLE_RESET_COLOR: &str = "\\x1b[0m";

pub struct MarkupOptions {
    pub color_mode: ColorMode,
    pub newline: bool,
    pub handle_escape: bool,
    pub no_binary_expansion: bool,
    pub logger: Logger,
}

impl Default for MarkupOptions {
    fn default() -> Self {
        Self {
            color_mode: ColorMode::Color256,
            newline: false,
            handle_escape: true,
            no_binary_expansion: false,
            logger: Logger::new(),
        }
    }
}

pub fn markup_text<T>(user_text: &str, options: T) -> String
where
    T: Into<MarkupOptions>,
{
    let opt = options.into();

    let mut buffer = String::new();
    let mut color_buffer = String::new();

    let mut state = MachineState::Normal;

    let chars: Vec<char> = user_text.chars().collect();
    let len = chars.len();

    let lookbehind_equals = |index: usize, char: char| -> bool {
        return chars.get(index - 1) == Some(&char);
    };

    let peek_equals = |index: usize, char: char| -> bool {
        return chars.get(index + 1) == Some(&char);
    };

    for (i, char) in chars.iter().enumerate() {
        match state {
            MachineState::Normal => {
                if *char == '\x1b' && peek_equals(i, '[') {
                    color_buffer.push('\x1b');
                    state = MachineState::ReadingAnsiCode;
                } else if *char == '[' {
                    if i + 1 < len && chars.get(i + 1) == Some(&'[') {
                        v_log!(opt.logger, "Escaping tag at index {}", i);
                        buffer.push('[');
                    }

                    state = MachineState::ReadingColor;
                    color_buffer.clear();
                } else {
                    buffer.push(*char);
                }
            }
            MachineState::ReadingColor => {
                if *char == '[' {
                    v_log!(
                        opt.logger,
                        "Found unexpected opening tag, ignoring. ({})",
                        *char
                    );
                    continue;
                }

                if lookbehind_equals(i, ']') && *char == '/' {
                    state = MachineState::ReadingReset;
                } else if *char == ']' && !(i >= 1 && lookbehind_equals(i, '[')) {
                    let color = resolve_color_code(
                        &color_buffer,
                        false,
                        opt.color_mode,
                        opt.no_binary_expansion,
                        opt.logger,
                    );
                    buffer.push_str(&color);
                    color_buffer.clear();
                    state = MachineState::Normal;
                } else {
                    color_buffer.push(*char);
                }
            }
            MachineState::ReadingReset => {
                if *char == ']' && !(i >= 1 && lookbehind_equals(i, '[')) {
                    buffer.push_str(if opt.no_binary_expansion {
                        READABLE_RESET_COLOR
                    } else {
                        RESET_COLOR
                    });
                } else {
                    v_log!(opt.logger, "Unexpected character {}, ignoring.", *char);
                }

                state = MachineState::Normal;
            }
            MachineState::ReadingAnsiCode => {
                color_buffer.push(*char);
                if *char >= '@' && *char <= '~' {
                    v_log!(opt.logger, "Found ANSI code: {:?}", color_buffer);
                    buffer.push_str(&color_buffer);
                    color_buffer.clear();
                    state = MachineState::Normal;
                } else if color_buffer.len() > 32 {
                    buffer.push_str(&color_buffer);
                    color_buffer.clear();
                    state = MachineState::Normal;
                }
            }
        }
    }

    if state == MachineState::ReadingColor {
        let color = resolve_color_code(
            &color_buffer,
            false,
            opt.color_mode,
            opt.no_binary_expansion,
            opt.logger,
        );
        buffer.push_str(&color);
    }

    if opt.handle_escape {
        expand_escape_codes(&buffer);
    }

    if opt.newline {
        buffer.push('\n');
    }

    return buffer;
}

fn resolve_color_code(
    raw_color: &str,
    resolving_background: bool,
    color_mode: ColorMode,
    no_binary_expansion: bool,
    logger: Logger,
) -> String {
    if raw_color.is_empty() || raw_color == "/" || color_mode == ColorMode::NoColor {
        return if no_binary_expansion {
            READABLE_RESET_COLOR.to_owned()
        } else {
            RESET_COLOR.to_owned()
        };
    }

    let mut color_string = raw_color;

    let mut final_color = String::new();
    if !resolving_background {
        let segments = color_string.split_once(" on ");
        match segments {
            Some(n) => {
                final_color =
                    resolve_color_code(n.1, true, color_mode, no_binary_expansion, logger);
                color_string = n.0;
            }
            None => {}
        }
    }

    v_log!(logger, "Resolving tag '{}'", color_string);

    let (style_str, color_str) = extract_color_information(color_string, logger);

    if !color_str.is_empty() && color_str != "_" {
        // get color
        if color_str.starts_with('#') && color_str.len() == 7 {
            if let Ok(i_color) = u32::from_str_radix(&color_str[1..], 16) {
                let hex_color = output_color_from_hex(i_color, resolving_background);
                v_log!(logger, "Hex color {}", hex_color);
                final_color.push_str(&hex_color);
            }
        } else if color_str.starts_with("rgb(") {
            if let Some((r, g, b)) = parse_rgb_manual(&color_str) {
                let rgb_color = output_color_from_rgb([r, g, b], resolving_background);
                v_log!(logger, "RGB color {}", rgb_color);
                final_color.push_str(&rgb_color);
            }
        } else {
            // get color by name
            match get_color_by_name(&color_str) {
                Some(color) => {
                    let named_color = output_color_from_hex(color, resolving_background);
                    v_log!(logger, "Using named color {}", named_color);
                    final_color.push_str(&named_color);
                }
                None => {
                    v_log!(logger, "Failed to find color '{}'", color_str);
                    return String::new();
                }
            }
        }
    }

    let mut color_assemble = format!("{};{}", style_str, final_color);
    clean_semicolons(&mut color_assemble);
    v_log!(logger, "Assembled color: {}", color_assemble);

    if resolving_background {
        v_log!(logger, "Final background color: {}", color_assemble);
        return color_assemble;
    }

    let final_color = format!(
        "{}[{}m",
        if no_binary_expansion { "\\x1b" } else { "\x1b" },
        color_assemble
    );
    v_log!(logger, "Final color: {:?}", final_color);
    return final_color;
}

fn extract_color_information(raw_color: &str, logger: Logger) -> (String, String) {
    let mut style_buffer = String::new();
    let mut color = String::new();

    for item in raw_color.split(" ") {
        match get_format_code_from_label(item) {
            Some(code) => {
                style_buffer.push_str(code);
            }
            None => {
                color = item.to_owned();
                v_log!(logger, "Possible color item '{}'", item);
            }
        }
    }

    if style_buffer.starts_with(';') {
        style_buffer.remove(0);
    }

    v_log!(logger, "Resolved style: {}", style_buffer);
    v_log!(logger, "Resolved color: {}", color);
    return (style_buffer, color);
}

pub fn split_rgb_int(color: u32) -> (u8, u8, u8) {
    let r: u8 = ((color >> 16) & 0xff) as u8;
    let g: u8 = ((color >> 8) & 0xff) as u8;
    let b: u8 = (color & 0xff) as u8;
    return (r, g, b);
}

#[allow(unused)]
fn string_hex_to_rgb(hex_color: &str) -> (u8, u8, u8) {
    let hex = &hex_color[1..];
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    return (r, g, b);
}

fn find_nearest_color(user_rgb: [u8; 3]) -> u32 {
    let mut min_distance_sq = u32::MAX;
    let mut nearest_hex = 0;

    // Pre-cast user RGB to u32 once
    let [ur, ug, ub] = [user_rgb[0] as u32, user_rgb[1] as u32, user_rgb[2] as u32];

    for &(_name, hex_color) in COLORS {
        let r = (hex_color >> 16) & 0xFF;
        let g = (hex_color >> 8) & 0xFF;
        let b = hex_color & 0xFF;

        let dr = ur.abs_diff(r);
        let dg = ug.abs_diff(g);
        let db = ub.abs_diff(b);

        let distance_sq = dr * dr + dg * dg + db * db;

        if distance_sq < min_distance_sq {
            min_distance_sq = distance_sq;
            nearest_hex = hex_color;
        }
    }

    return nearest_hex;
}

fn parse_rgb_manual(s: &str) -> Option<(u8, u8, u8)> {
    let trimmed = s.strip_prefix("rgb(")?.strip_suffix(')')?;

    let mut parts = trimmed.split(',');

    let r = parts.next()?.trim().parse().ok()?;
    let g = parts.next()?.trim().parse().ok()?;
    let b = parts.next()?.trim().parse().ok()?;

    Some((r, g, b))
}

fn output_color_from_hex(color: u32, background: bool) -> String {
    let (r, g, b) = split_rgb_int(color);
    return output_color_from_rgb([r, g, b], background);
}

fn output_color_from_rgb(colors: [u8; 3], background: bool) -> String {
    let color_type = if background { 48 } else { 38 };
    let (r, g, b) = split_rgb_int(find_nearest_color(colors));
    return format!(";{};2;{};{};{}", color_type, r, g, b);
}

fn clean_semicolons(text: &mut String) {
    let mut last_was_semi = false;

    text.retain(|c| {
        let is_semi = c == ';';
        let keep = !(is_semi && last_was_semi);
        last_was_semi = is_semi;
        return keep;
    });

    let trimmed = text.trim_matches(';').to_string();
    text.clear();
    text.push_str(&trimmed);
}

fn expand_escape_codes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
        let mut matched = false;
        let remaining = &input[i..];

        for (key, val) in CONTROL_CHARS {
            if remaining.starts_with(key) {
                result.push_str(val);
                i += key.len();
                matched = true;
                break;
            }
        }

        if !matched {
            if let Some(c) = remaining.chars().next() {
                result.push(c);
                i += c.len_utf8();
            } else {
                break;
            }
        }
    }

    return result;
}

#[derive(Debug, Default, PartialEq)]
enum MachineState {
    #[default]
    Normal,
    ReadingColor,
    ReadingReset,
    ReadingAnsiCode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_to_rgb() {
        let split = split_rgb_int(0x123456);
        assert_eq!(
            split,
            (0x12u8, 0x34u8, 0x56u8),
            "RGB channels did not match expected values."
        );
    }

    #[test]
    fn text_string_to_rgb() {
        let split = string_hex_to_rgb("#33ffaa");
        assert_eq!(
            split,
            (0x33u8, 0xffu8, 0xaau8),
            "RGB channels did not match expected values."
        );
    }

    #[test]
    fn test_nearest_color_search() {
        // Should return 215,95,135, hotpink3_1
        let nearest_color = find_nearest_color([0xd7u8, 0x5fu8, 0x86u8]);
        assert_eq!(
            nearest_color, 0xd75f87,
            "Unexpected nearest color {nearest_color}, expected {}",
            0xd75f87
        );
    }

    #[test]
    fn test_escape_expansion() {
        let keys: String = CONTROL_CHARS.iter().map(|s| s.0).collect();
        eprintln!("Keys: {}", keys);
        let expanded_string = expand_escape_codes(&keys);

        let values: String = CONTROL_CHARS.iter().map(|s| s.1).collect();
        assert_eq!(expanded_string, values);
    }

    /*
     * Markup tests
     */

    fn test_markup_text_passthrough() {
        let s = MarkupOptions::default();
        let result = markup_text("Hello, World!", s);
    }
}
