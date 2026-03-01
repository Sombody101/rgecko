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

struct VisitorState {
    /* Used to store ANSI styles */
    style_buffer: String,
    /* Used for formatting ANSI codes */
    ansi_buffer: String,
    /* Used for assembling colors. This is the only one that can and will contain binary (0x1b) */
    scratch: String,
}

impl VisitorState {
    fn new() -> Self {
        Self {
            style_buffer: String::with_capacity(32),
            ansi_buffer: String::with_capacity(32),
            scratch: String::with_capacity(64),
        }
    }
}

pub fn markup_text<T>(user_text: &str, options: T) -> String
where
    T: Into<MarkupOptions>,
{
    if user_text.is_empty() {
        return String::new();
    }

    let opt = options.into();

    let mut visitor_state = VisitorState::new();
    let mut buffer = String::with_capacity(user_text.len());
    let mut color_buffer = String::with_capacity(32);

    let mut machine_state = MachineState::Normal;

    let chars: Vec<char> = user_text.chars().collect();
    let len = chars.len();

    let lookbehind_equals = |index: usize, char: char| -> bool {
        return index != 0 && chars.get(index - 1) == Some(&char);
    };

    let peek_equals = |index: usize, char: char| -> bool {
        return index != len - 1 && chars.get(index + 1) == Some(&char);
    };

    for (i, &char) in chars.iter().enumerate() {
        match machine_state {
            MachineState::Normal => {
                if char == '\x1b' && peek_equals(i, '[') {
                    color_buffer.push('\x1b');
                    machine_state = MachineState::ReadingAnsiCode;
                } else if char == '[' {
                    if peek_equals(i, '[') {
                        v_log!(opt.logger, "Escaping tag at index {}", i);
                        buffer.push('[');
                        machine_state = MachineState::Skip;
                        continue;
                    }

                    machine_state = MachineState::ReadingColor;
                    color_buffer.clear();
                } else {
                    buffer.push(char);
                }
            }
            MachineState::ReadingColor => {
                if char == '[' {
                    v_log!(opt.logger, "Found unexpected opening tag, ignoring.");
                    continue;
                }

                if lookbehind_equals(i, ']') && char == '/' {
                    machine_state = MachineState::ReadingReset;
                } else if char == ']' && !(i >= 1 && lookbehind_equals(i, '[')) {
                    resolve_color_code(
                        &color_buffer,
                        false,
                        opt.color_mode,
                        opt.no_binary_expansion,
                        &mut visitor_state,
                        &mut buffer,
                        opt.logger,
                    );
                    machine_state = MachineState::Normal;
                } else {
                    color_buffer.push(char);
                }
            }
            MachineState::ReadingReset => {
                if char == ']' && !(i >= 1 && lookbehind_equals(i, '[')) {
                    buffer.push_str(if opt.no_binary_expansion {
                        READABLE_RESET_COLOR
                    } else {
                        RESET_COLOR
                    });
                } else {
                    v_log!(opt.logger, "Unexpected character {}, ignoring.", char);
                }

                machine_state = MachineState::Normal;
            }
            MachineState::ReadingAnsiCode => {
                color_buffer.push(char);
                if char >= '@' && char <= '~' {
                    v_log!(opt.logger, "Found ANSI code: {:?}", color_buffer);
                    buffer.push_str(&color_buffer);
                    color_buffer.clear();
                    machine_state = MachineState::Normal;
                } else if color_buffer.len() > 32 {
                    buffer.push_str(&color_buffer);
                    color_buffer.clear();
                    machine_state = MachineState::Normal;
                }
            }
            MachineState::Skip => {
                machine_state = MachineState::Normal;
            }
        }
    }

    if machine_state == MachineState::ReadingColor {
        resolve_color_code(
            &color_buffer,
            false,
            opt.color_mode,
            opt.no_binary_expansion,
            &mut visitor_state,
            &mut buffer,
            opt.logger,
        );
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
    color_text: &str,
    resolving_background: bool,
    color_mode: ColorMode,
    no_binary_expansion: bool,
    visitor_state: &mut VisitorState,
    output: &mut String,
    logger: Logger,
) {
    visitor_state.scratch.clear();

    if color_text.is_empty() || color_text == "/" || color_mode == ColorMode::NoColor {
        if no_binary_expansion {
            output.push_str(READABLE_RESET_COLOR)
        } else {
            output.push_str(RESET_COLOR)
        };

        return;
    }

    let fallback_index = output.len();
    let mut primary_color = color_text;

    if !resolving_background {
        if let Some((fg, bg)) = color_text.split_once(" on ") {
            resolve_color_code(
                bg,
                true,
                color_mode,
                no_binary_expansion,
                visitor_state,
                output,
                logger,
            );
            primary_color = fg;
        }
    }

    v_log!(logger, "Resolving tag '{}'", primary_color);

    let style_buffer = &mut visitor_state.style_buffer;
    let ansi_buffer = &mut visitor_state.ansi_buffer;
    extract_color_information(primary_color, style_buffer, ansi_buffer, logger);

    /*
     * output should NOT be modified above this comment, it could lead to output corruption
     */

    if !ansi_buffer.is_empty() && ansi_buffer != "_" {
        if ansi_buffer.starts_with('#')
            && let 4 | 7 = ansi_buffer.len()
        {
            let hex_content = &ansi_buffer[1..];
            if let Ok(i_color) = u32::from_str_radix(hex_content, 16) {
                let final_color = if hex_content.len() == 3 {
                    let r = (i_color >> 8) & 0xF;
                    let g = (i_color >> 4) & 0xF;
                    let b = i_color & 0xF;
                    (r << 20) | (r << 16) | (g << 12) | (g << 8) | (b << 4) | b
                } else {
                    i_color
                };

                // hexadecimal
                ansi_buffer.clear();
                get_color_from_hex(final_color, resolving_background, ansi_buffer);
            }
        } else if ansi_buffer.starts_with("rgb(") {
            // RGB
            if let Some((r, g, b)) = parse_rgb_manual(&ansi_buffer) {
                ansi_buffer.clear();
                get_color_from_rgb([r, g, b], resolving_background, ansi_buffer);
            }
        } else {
            // named color
            if let Some(color) = get_color_by_name(&ansi_buffer) {
                ansi_buffer.clear();
                get_color_from_hex(color, resolving_background, ansi_buffer);
            } else {
                v_log!(logger, "Failed to find color '{}'", ansi_buffer);
                return output.truncate(fallback_index);
            }
        }
    } else if ansi_buffer == "_" {
        ansi_buffer.clear();
    }

    let scratch = &mut visitor_state.scratch;
    v_log!(logger, "Final style: {:?}", style_buffer);
    scratch.push_str(style_buffer);
    v_log!(logger, "ANSI color: {:?}", ansi_buffer);
    scratch.push_str(ansi_buffer);

    if resolving_background {
        // background color data is already in the buffers
        // the below pushes are for finalizing the ANSI command
        return;
    }

    output.push_str(if no_binary_expansion {
        "\\x1b["
    } else {
        "\x1b["
    });
    output.push_str(scratch.strip_prefix(';').unwrap_or(&scratch));
    output.push('m');
}

fn extract_color_information(
    raw_color: &str,
    style_out: &mut String,
    color_out: &mut String,
    logger: Logger,
) {
    style_out.clear();
    color_out.clear();

    for item in raw_color.split(' ') {
        v_log!(logger, "Working on '{}'", item);
        if let Some(code) = get_format_code_from_label(item) {
            style_out.push_str(code);
        } else {
            color_out.clear();
            color_out.push_str(item);
        }
    }

    v_log!(logger, "Resolved style: {}", style_out);
    v_log!(logger, "Resolved color: {}", color_out);
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

fn get_color_from_hex(color: u32, background: bool, buffer: &mut String) {
    let (r, g, b) = split_rgb_int(color);
    get_color_from_rgb([r, g, b], background, buffer);
}

fn get_color_from_rgb(colors: [u8; 3], background: bool, buffer: &mut String) {
    let color_type = if background { 48 } else { 38 };
    let (r, g, b) = split_rgb_int(find_nearest_color(colors));
    use std::fmt::Write;
    let _ = write!(buffer, ";{};2;{};{};{}", color_type, r, g, b);
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
    Skip,
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
