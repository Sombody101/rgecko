use crate::argparse::parser::{CliConfig, ExtraMode};
use crate::colors::{ansicodes::FORMAT_CODES, colorsheet::COLORS};
use std::process::exit;

const SAMPLE_TEXT: &str = "Hello, World!";
const VERSION: &str = match option_env!("APP_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

struct StyleState<'a> {
    output: &'a mut String,
    tag_buf: String,
    code_buf: String,
}

pub fn handle_cli_extras(config: &CliConfig, output: &mut String) -> bool {
    match config.extras {
        ExtraMode::PrintHelp => {
            print_help_info();
            return true;
        }
        ExtraMode::PrintVersion => {
            print_version_info();
            return true;
        }
        ExtraMode::ListColors | ExtraMode::ListStyles | ExtraMode::ListColorsAsBackground => {
            list_styling_examples(&config.text_input, output, config.extras, config);
            return true;
        }
        ExtraMode::None => {}
    }

    false
}

fn list_styling_examples(
    user_text: &str,
    output: &mut String,
    mode: ExtraMode,
    config: &CliConfig,
) {
    let sample_text = if user_text.is_empty() {
        SAMPLE_TEXT
    } else {
        user_text
    };

    let output_buffer_size = calculate_output_buffer_size(user_text.len(), mode);
    v_log!(config.logger, "Buff size: {}", output_buffer_size);
    output.reserve(output_buffer_size);

    let mut style_state = StyleState {
        output,
        tag_buf: String::with_capacity(64),
        code_buf: String::with_capacity(64),
    };

    match mode {
        ExtraMode::ListColors => list_colors(sample_text, &mut style_state, false),
        ExtraMode::ListColorsAsBackground => list_colors(sample_text, &mut style_state, true),
        ExtraMode::ListStyles => list_styles(sample_text, &mut style_state),
        _ => {
            v_log!(config.logger, "Unknown extra mode: {:?}", mode);
        }
    };
}

fn list_colors(user_text: &str, state: &mut StyleState, as_background: bool) {
    list_items(COLORS, state, |color, state| {
        format_listing(ColorEntry::from(*color), state, as_background, user_text);
    })
}

fn list_styles(user_text: &str, state: &mut StyleState) {
    list_items(FORMAT_CODES, state, |style, state| {
        format_listing(StyleEntry::from(*style), state, false, user_text)
    })
}

fn list_items<T, F>(items: &[T], state: &mut StyleState, mut processor: F)
where
    F: FnMut(&T, &mut StyleState) -> (),
{
    for item in items {
        processor(item, state);
    }
}

fn format_listing<T: GeckoExample>(
    item: T,
    state: &mut StyleState,
    is_background: bool,
    sample_text: &str,
) {
    state.code_buf.clear();
    item.code_display(&mut state.code_buf);

    state.tag_buf.clear();
    if is_background {
        state.tag_buf.push_str("_ on ");
    }
    item.tag(&mut state.tag_buf);

    use std::fmt::Write;
    let _ = write!(
        state.output,
        "({}) {:<width$} [{}]{}[/]\n",
        state.code_buf,
        item.label(),
        state.tag_buf,
        sample_text,
        width = item.width()
    );
}

fn print_help_info() {
    print!(
        r#"Usage: gecko [OPTIONS] [TEXT]...

Arguments:
  [TEXT]...  Text to display with markup (color)

Options:
  -n, --no-newline      Do not print a newline at the end
  -e, --handle-escape   Expand escape sequences
  -c, --force-color     Force color output (256-color) even if not detected
  -C, --no-color        Disable color output even if detected
  -m, --no-markup       Do not resolve markup sequences
  -i, --interactive     Forward output to less
      --listc           List possible colors (Sample text can be appended, defaults to "Hello, World!)
      --listcb          List possible colors as backgrounds
      --lists           List possible styles and their ANSI codes
      --help            Print help
      --version         Print version
  
rgecko v{VERSION}
"#
    );
}

fn print_version_info() {
    println!("rgecko v{VERSION}");
}

fn calculate_output_buffer_size(sample_length: usize, mode: ExtraMode) -> usize {
    let row_overhead = 15;
    let label_width = 20;
    let code_width = 10;

    let (tag_padding, count) = match mode {
        ExtraMode::ListColors => (15, 256),
        ExtraMode::ListColorsAsBackground => (30, 256),
        ExtraMode::ListStyles => (10, 10),
        _ => (5, 1),
    };

    let bytes_per_line = row_overhead + label_width + code_width + tag_padding + sample_length;

    let total_estimate = (bytes_per_line * count) + 1024;

    if total_estimate >= (500 * 1024 * 1024) {
        eprintln!("Input text too large0!");
        exit(1);
    }

    total_estimate
}

trait GeckoExample {
    fn label(&self) -> &str;
    fn code_display(&self, output: &mut String);
    fn tag(&self, output: &mut String);
    fn width(&self) -> usize;
}

struct ColorEntry<'a> {
    name: &'a str,
    code: u32,
}

impl<'a> GeckoExample for ColorEntry<'a> {
    fn label(&self) -> &str {
        &self.name
    }
    fn code_display(&self, output: &mut String) {
        use std::fmt::Write;
        let _ = write!(output, "#{:06x}", self.code);
    }
    fn tag(&self, output: &mut String) {
        use std::fmt::Write;
        let _ = write!(output, "#{:06x}", self.code);
    }
    fn width(&self) -> usize {
        17
    }
}

impl<'a> From<(&'a str, u32)> for ColorEntry<'a> {
    fn from(pair: (&'a str, u32)) -> Self {
        ColorEntry {
            name: pair.0,
            code: pair.1,
        }
    }
}

struct StyleEntry<'a> {
    name: &'a str,
    ansi_code: &'a str,
}

impl<'a> GeckoExample for StyleEntry<'a> {
    fn label(&self) -> &str {
        &self.name
    }
    fn code_display(&self, output: &mut String) {
        use std::fmt::Write;
        let _ = write!(output, "ANSI: {:>2}", self.ansi_code);
    }
    fn tag(&self, output: &mut String) {
        use std::fmt::Write;
        let _ = write!(output, "{}", self.name);
    }
    fn width(&self) -> usize {
        11
    }
}

impl<'a> From<(&'a str, &'a str)> for StyleEntry<'a> {
    fn from(pair: (&'a str, &'a str)) -> Self {
        StyleEntry {
            name: pair.0,
            ansi_code: pair.1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extras() {}
}
