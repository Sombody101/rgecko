use crate::argparse::parser::{CliConfig, ExtraMode};
use crate::colors::transform::MarkupOptions;
use crate::colors::{ansicodes::FORMAT_CODES, colorsheet::COLORS, transform::markup_text};

const SAMPLE_TEXT: &str = "Hello, World!";

pub fn handle_cli_extras(config: CliConfig) -> bool {
    match config.extras {
        ExtraMode::PrintHelp => {
            print_help_info();
            return true;
        }
        ExtraMode::ListColors | ExtraMode::ListStyles | ExtraMode::ListColorsAsBackground => {
            list_styling_examples(&config.text_input, config.extras, config);
            return true;
        }
        ExtraMode::None => {}
    }

    return false;
}

fn list_styling_examples(user_text: &str, mode: ExtraMode, config: CliConfig) {
    let sample_text = if user_text.is_empty() {
        SAMPLE_TEXT
    } else {
        user_text
    };

    let output = match mode {
        ExtraMode::ListColors => list_colors(sample_text, false, config),
        ExtraMode::ListColorsAsBackground => list_colors(sample_text, true, config),
        ExtraMode::ListStyles => list_styles(sample_text, config),
        _ => String::new(),
    };

    print!("{}", output);
    println!("This bitch is {} chars", output.len());
}

fn list_colors(user_text: &str, as_background: bool, config: CliConfig) -> String {
    list_items(COLORS, |color| {
        format_listing(ColorEntry::from(*color), as_background, user_text, config)
    })
}

fn list_styles(user_text: &str, config: CliConfig) -> String {
    list_items(FORMAT_CODES, |style| {
        format_listing(StyleEntry::from(*style), false, user_text, config)
    })
}

fn list_items<T, F>(items: &[T], mut processor: F) -> String
where
    F: FnMut(&T) -> String,
{
    let mut buffer = String::with_capacity(16300);

    for item in items {
        buffer.push_str(&processor(item));
    }

    return buffer;
}

fn format_listing<T: GeckoExample>(
    item: T,
    is_background: bool,
    sample_text: &str,
    config: CliConfig,
) -> String {
    let tag = item.tag();

    let final_tag = if is_background {
        format!("_ on {tag}")
    } else {
        tag
    };

    let formatted_string = format!(
        "({}) {:<width$} [{}]{}[/]",
        item.code_display(),
        item.label(),
        final_tag,
        sample_text,
        width = item.width()
    );

    v_log!(config.logger, "{}", formatted_string);

    let options = MarkupOptions {
        color_mode: config.color_mode,
        newline: config.newline,
        handle_escape: config.handle_escape,
        no_binary_expansion: config.no_binary_expansion,
        logger: config.logger,
    };

    let processed_string = markup_text(&formatted_string, options);

    return processed_string;
}

fn print_help_info() {
    print!(
        r#"usage: gecko [[options...]] [[text..]]
Display text with markup (color), speedily and efficiently.

Stops parsing options after standalone '--' is read.

    Options:
      -n:	No newline when printing output text

      -c:	Force color output even when not detected as supported (256 color)

      -C:	Force no color output even when detected as supported

      -M:	Do not resolve markup sequences

      --listc:	List possible colors (listc[olors])
          (Add text after to change sample text, markup is still parsed)

      --listcb:	List possible colors as backgrounds (listc[olor]b[ackgrounds])

      --lists:	 List possible styles and their ANSI codes
"#
    );
}

trait GeckoExample {
    fn label(&self) -> &str;
    fn code_display(&self) -> String;
    fn tag(&self) -> String;
    fn width(&self) -> usize;
}

struct ColorEntry {
    name: String,
    code: u32,
}

impl GeckoExample for ColorEntry {
    fn label(&self) -> &str {
        &self.name
    }
    fn code_display(&self) -> String {
        format!("#{:06x}", self.code)
    }
    fn tag(&self) -> String {
        format!("#{:06x}", self.code)
    }
    fn width(&self) -> usize {
        17
    }
}

impl From<(&str, u32)> for ColorEntry {
    fn from(pair: (&str, u32)) -> Self {
        return ColorEntry {
            name: pair.0.to_string(),
            code: pair.1,
        };
    }
}

struct StyleEntry {
    name: String,
    ansi_code: String,
}

impl GeckoExample for StyleEntry {
    fn label(&self) -> &str {
        &self.name
    }
    fn code_display(&self) -> String {
        format!("ANSI: {:>2}", self.ansi_code)
    }
    fn tag(&self) -> String {
        self.name.clone()
    }
    fn width(&self) -> usize {
        11
    }
}

impl From<(&str, &str)> for StyleEntry {
    fn from(pair: (&str, &str)) -> Self {
        return StyleEntry {
            name: pair.0.to_string(),
            ansi_code: pair.1.to_string(),
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extras() {}
}
