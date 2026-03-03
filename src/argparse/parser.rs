use crate::colors::terms::get_color_support;
use crate::{logger::Logger, v_log};
use std::env;

pub struct CliConfig {
    pub color_mode: ColorMode,

    pub handle_escape: bool,

    pub newline: bool,

    // Just act like echo when true
    pub no_markup: bool,

    pub text_input: String,

    pub extras: ExtraMode,

    pub logger: Logger,

    pub no_binary_expansion: bool,

    // Forwards text to less. Mainly for listc and listcb, but will also work for normal output if its large enough
    pub interactive: bool,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum ColorMode {
    #[default]
    Default,
    NoColor,
    Color256,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum ExtraMode {
    #[default]
    None,
    PrintHelp,
    PrintVersion,
    ListColors,
    ListColorsAsBackground,
    ListStyles,
}

pub fn parse_args<I, T>(args: I) -> CliConfig
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let mut config = CliConfig {
        color_mode: ColorMode::Default,
        handle_escape: false,
        newline: true,
        no_markup: false,
        text_input: String::new(),
        extras: ExtraMode::None,
        logger: create_logger(),
        no_binary_expansion: false,
        interactive: false,
    };

    let mut iter = args.into_iter().peekable();
    if iter.peek().is_none() {
        return config;
    }

    let (min_size, _) = iter.size_hint();
    v_log!(config.logger, "Using min-size: {:?}", min_size);
    config.text_input.reserve(min_size);
    loop_arguments(&mut iter, &mut config);

    if config.color_mode == ColorMode::Default {
        config.color_mode = get_color_support();
    }

    config
}

fn loop_arguments<I, T>(iter: &mut I, config: &mut CliConfig)
where
    I: Iterator<Item = T>,
    T: AsRef<str>,
{
    while let Some(arg) = iter.next() {
        let word = arg.as_ref();

        if word == "--" {
            for (i, x) in iter.enumerate() {
                if i > 0 || !config.text_input.is_empty() {
                    config.text_input.push(' ');
                }
                config.text_input.push_str(x.as_ref());
            }
            break;
        }

        if word.starts_with("--") {
            let trimmed_switch = &word[2..];
            v_log!(config.logger, "Switch: {}", trimmed_switch);
            resolve_switch(trimmed_switch, config);
        } else if word.starts_with('-') {
            let trimmed_switch = &word[1..];
            v_log!(config.logger, "Shorthand group: {}", trimmed_switch);
            for char in trimmed_switch.chars() {
                resolve_shorthand_switch(char, config);
            }
        } else {
            v_log!(config.logger, "Pushing non-arg: {word}");
            if !config.text_input.is_empty() {
                config.text_input.push(' ');
            }
            config.text_input.push_str(word);
        }
    }
}

fn resolve_shorthand_switch(switch: char, config: &mut CliConfig) {
    match switch {
        'n' => config.newline = false,
        'e' => config.handle_escape = true,
        'c' => config.color_mode = ColorMode::Color256,
        'C' => config.color_mode = ColorMode::NoColor,
        'm' => config.no_markup = true,
        'i' => config.interactive = true,
        _ => {}
    };
}

fn resolve_switch(word: &str, config: &mut CliConfig) {
    match word {
        "help" => config.extras = ExtraMode::PrintHelp,
        "version" => config.extras = ExtraMode::PrintVersion,
        "listc" => config.extras = ExtraMode::ListColors,
        "listcb" => config.extras = ExtraMode::ListColorsAsBackground,
        "lists" => config.extras = ExtraMode::ListStyles,
        "no-markup" => config.no_markup = true,
        "interactive" => config.interactive = true,
        "nobexp" => config.no_binary_expansion = true,
        "verbose" => config.logger.verbose = true,
        &_ => {}
    };
}

fn create_logger() -> Logger {
    Logger {
        verbose: match env::var("VERBOSE_LOG") {
            Ok(_) => true,
            Err(_) => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_parse() {
        let config = parse_args(&["-nc", "Hello,", "--listc", "World!", "--", "--lists", "!"]);
        assert_eq!(
            config.text_input, "Hello, World! --lists !",
            "Invalid text scanning."
        );
        assert_eq!(config.newline, false, "Newline flag.");
        assert_eq!(config.color_mode, ColorMode::Color256, "Color mode flag.");
    }
}
