use crate::argparse::parser::ColorMode;
use crate::colors::ansicodes::{AnsiCommand, Color, Layer, Style};
use crate::colors::colorsheet::{COLORS, get_color_by_name};
use crate::colors::transform::{MachineState, MarkupOptions, split_rgb_int};
use crate::logger::Logger;
use crate::v_log;

pub fn markup_text(text: &str, opt: &MarkupOptions) -> String {
    let logger = opt.logger;

    let mut output = String::with_capacity(text.len() * 3);

    let mut state = MachineState::Normal;

    let mut segment_start: usize = 0;
    let mut segment_len: usize = 0;

    let bytes = text.as_bytes();
    let len = bytes.len();

    let peek_eq = |index: usize, b: u8| -> bool { index + 1 < len && bytes[index + 1] == b };

    let prev_eq = |index: usize, b: u8| -> bool { index > 0 && bytes[index - 1] == b };

    let mut idx = usize::MAX;
    loop {
        idx = idx.wrapping_add(1);
        if idx >= len {
            break;
        }

        let char = bytes[idx] as char;

        match state {
            MachineState::Normal => {
                if char == '[' {
                    if !peek_eq(idx, b'[') {
                        segment_start = idx + 1;
                        state = MachineState::ReadingColor;
                        continue;
                    }

                    v_log!(logger, "Escaping tag at index {idx}");
                    output.push('[');
                    idx += 1;
                } else if char == '\\' && opt.handle_escape {
                    v_log!(logger, "Unable to handle escape codes.");
                    //state = MachineState::ReadingEscapeCode;
                } else if char == '\x1b' && (peek_eq(idx, b'[') || peek_eq(idx, b'(')) {
                    // Start tag
                    segment_start = idx;
                    state = MachineState::ReadingAnsiCode;
                } else {
                    output.push(char);
                }
            }

            MachineState::ReadingColor => {
                if char == '[' {
                    v_log!(opt.logger, "Unexpected opening tag, ignoring.");
                    continue;
                }

                if char == ' ' && prev_eq(idx, b' ') {
                    // This eats any random gaps like "[ blue   on white ]"
                    continue;
                }

                // Reset tag: "[/]"
                if char == '/' && prev_eq(idx, b'[') {
                    state = MachineState::ReadingReset;
                    continue;
                }

                /* Finishes tags, but ignores cases like "[]" */
                if char == ']' {
                    state = MachineState::Normal;

                    if prev_eq(idx, b'[') {
                        v_log!(logger, "Unexpected empty tag, ignoring.");
                        continue;
                    }

                    let tag = &text[segment_start..(segment_start + segment_len)];
                    resolve_color_tag(tag, &mut output, opt);
                    segment_start = 0;
                    segment_len = 0;
                    continue;
                }

                segment_len += 1;
            }

            MachineState::ReadingReset => {
                if char == ']' && !(idx >= 1 && prev_eq(idx, b'[')) {
                    let reset = if opt.no_binary_expansion {
                        READABLE_RESET_COLOR
                    } else {
                        RESET_COLOR
                    };

                    output.push_str(reset);
                } else {
                    v_log!(logger, "Unexpected character {}, ignoring.", char)
                }

                state = MachineState::Normal;
            }

            MachineState::ReadingEscapeCode => {
                v_log!(logger, "Unable to handle escape codes.");
                state = MachineState::Normal;
            }

            MachineState::ReadingAnsiCode => {
                segment_len += 1;
                if char >= '@' && char <= '~' {
                    v_log!(
                        logger,
                        "Found ANSI code: {:?}",
                        &text[segment_start..segment_len]
                    )
                } else if segment_len > 32 {
                    output.push_str(&text[segment_start..(segment_start + segment_len)]);
                    state = MachineState::Normal;
                }
            }

            MachineState::Skip => {
                v_log!(logger, "Skip MachineState reached.")
            }
        }
    }

    output
}

const RESET_COLOR: &str = "\x1b[0m";
const READABLE_RESET_COLOR: &str = "\\x1b[0m";

fn resolve_color_tag(color_text: &str, output: &mut String, opt: &MarkupOptions) {
    if color_text.is_empty() || color_text == "/" || opt.color_mode == ColorMode::NoColor {
        if opt.no_binary_expansion {
            output.push_str(READABLE_RESET_COLOR)
        } else {
            output.push_str(RESET_COLOR)
        };

        return;
    }

    let ansi_prefix = if opt.no_binary_expansion {
        "\\x1b["
    } else {
        "\x1b["
    };

    output.push_str(ansi_prefix);

    let fallback_index = output.len();

    if let Some((fg, bg)) = color_text.split_once(" on ") {
        resolve_color_segment(bg, true, output, opt);

        if fg != "_" {
            output.push(';');
            resolve_color_segment(fg, false, output, opt);
        }
    } else {
        resolve_color_segment(color_text, false, output, opt);
    }

    if output.len() == fallback_index {
        output.truncate(fallback_index - ansi_prefix.len());
        return;
    }

    output.push('m');
}

fn resolve_color_segment(segment: &str, bg: bool, output: &mut String, opt: &MarkupOptions) {
    v_log!(opt.logger, "Resolving tag [{segment:?}]");

    if let Some(command) = extract_color_information(segment, &opt.logger) {
        command.build_into(
            if bg {
                Layer::Background
            } else {
                Layer::Foreground
            },
            output,
        );
    };
}

fn extract_color_information(style_segment: &str, logger: &Logger) -> Option<AnsiCommand> {
    let mut style = Style::default();
    let mut color_str: &str = "";

    for item in style_segment.split(' ') {
        if let Some(flag) = style.set_flag(item) {
            v_log!(logger, "Style flag {item:?} -> {flag:?}");
        } else {
            color_str = item;
        }
    }

    // color lookup is deferred since there's many more colors to check
    let color: Option<Color> = if color_str != "_" && !color_str.is_empty() {
        resolve_color_string(color_str, logger)
    } else {
        None
    };

    v_log!(logger, "Resolved [style:{style:?}, color:{color:?}]");

    Some(AnsiCommand { style, color })
}

fn resolve_color_string(color: &str, logger: &Logger) -> Option<Color> {
    if color.starts_with('#') && matches!(color.len(), 4 | 7) {
        // hexadecimal
        let hex_content = &color[1..];
        if let Ok(i_color) = u32::from_str_radix(hex_content, 16) {
            let final_color = if hex_content.len() == 3 {
                let r = (i_color >> 8) & 0xF;
                let g = (i_color >> 4) & 0xF;
                let b = i_color & 0xF;
                (r << 20) | (r << 16) | (g << 12) | (g << 8) | (b << 4) | b
            } else {
                i_color
            };

            let c = get_color_from_hex(final_color);
            return Some(c);
        } else {
            v_log!(logger, "Invalid hex format, skipping color");
            return None;
        }
    } else if color.starts_with("rgb(") {
        // RGB
        if let Some((r, g, b)) = parse_rgb_manual(color) {
            let c = get_color_from_rgb([r, g, b]);
            return Some(c);
        } else {
            v_log!(logger, "Invalid RGB format, skipping color");
            return None;
        }
    }

    // named color
    if let Some(i_color) = get_color_by_name(&color) {
        let c = get_color_from_hex(i_color);
        Some(c)
    } else {
        v_log!(logger, "Nonexistent color {color:?}");
        None
    }
}

fn get_color_from_hex(color: u32) -> Color {
    let (r, g, b) = split_rgb_int(color);
    get_color_from_rgb([r, g, b])
}

fn get_color_from_rgb(colors: [u8; 3]) -> Color {
    let [r, g, b] = find_nearest_rgb(colors);

    Color { r, g, b }
}

fn parse_rgb_manual(s: &str) -> Option<(u8, u8, u8)> {
    let trimmed = s.strip_prefix("rgb(")?.strip_suffix(')')?;

    let mut parts = trimmed.split(',');

    let r = parts.next()?.trim().parse().ok()?;
    let g = parts.next()?.trim().parse().ok()?;
    let b = parts.next()?.trim().parse().ok()?;

    Some((r, g, b))
}

fn find_nearest_rgb(user_rgb: [u8; 3]) -> [u8; 3] {
    let mut min_distance_sq = u32::MAX;
    let mut nearest_rgb = [0u8; 3];

    let ur = user_rgb[0] as i32;
    let ug = user_rgb[1] as i32;
    let ub = user_rgb[2] as i32;

    for &(_name, hex) in COLORS {
        let r = ((hex >> 16) & 0xFF) as u8;
        let g = ((hex >> 8) & 0xFF) as u8;
        let b = (hex & 0xFF) as u8;

        let dr = ur - (r as i32);
        let dg = ug - (g as i32);
        let db = ub - (b as i32);

        let distance_sq = (dr * dr + dg * dg + db * db) as u32;

        if distance_sq < min_distance_sq {
            min_distance_sq = distance_sq;
            nearest_rgb = [r, g, b];

            if distance_sq == 0 {
                break;
            }
        }
    }

    nearest_rgb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_to_rgb() {
        const EXPECTED: (u8, u8, u8) = (0x12u8, 0x34u8, 0x56u8);
        let split = split_rgb_int(0x123456);
        assert_eq!(split, EXPECTED);
    }

    #[test]
    fn test_nearest_color_search_passthrough() {
        const EXPECTED: [u8; 3] = [0xd7u8, 0x5fu8, 0x87u8];

        // Should return 215,95,135, hotpink3_1
        let nc = find_nearest_rgb(EXPECTED);
        assert_eq!(nc, EXPECTED,);
    }

    #[test]
    fn test_nearest_color_search() {
        const EXPECTED: [u8; 3] = [0xffu8, 0x00u8, 0xffu8];

        let nc = find_nearest_rgb([0xffu8 - 2, 0x00u8 + 3, 0xffu8 - 1]);
        assert_eq!(nc, EXPECTED,);
    }

    /*
     * Markup tests
     */

    #[test]
    fn test_markup_rendering() {
        let options = MarkupOptions::default();

        let cases = [
            ("Hello, World!", "Hello, World!"),
            ("[blue]Hello[/]", "\x1b[38;2;0;0;255mHello\x1b[0m"),
            ("[_ on black]Hello[/]", "\x1b[48;2;0;0;0mHello\x1b[0m"),
            ("[blinking dunderlined]Hello[/]", "\x1b[5;21mHello\x1b[0m"),
            (
                "[bold dim italic underlined blinking fblinking swap striked dunderlined overlined]hello[/]",
                "\x1b[1;2;3;4;5;6;7;9;21;53mhello\x1b[0m",
            ),
            (
                "[blue white green red green on white blue black green black]hello[/]",
                "\x1b[48;2;0;0;0;38;2;0;128;0mhello\x1b[0m",
            ),
        ];

        for (input, expected) in cases {
            let result = markup_text(input, &options);
            assert_eq!(result, expected, "Failed for input: {input:?}");
        }
    }
}
