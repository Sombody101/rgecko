use crate::colors::terms::get_color_support;
use crate::{logger::Logger, v_log};
use std::borrow::Cow;
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
    ListColors,
    ListColorsAsBackground,
    ListStyles,
}

pub fn parse_args(args: &[&str]) -> CliConfig {
    let mut config = CliConfig {
        color_mode: ColorMode::Default,
        handle_escape: false,
        newline: true,
        no_markup: false,
        text_input: String::new(),
        extras: ExtraMode::None,
        logger: create_logger(),
        no_binary_expansion: false,
    };

    if args.is_empty() {
        return config;
    }

    loop_arguments(args, &mut config);

    let color_support = get_color_support();
    if config.color_mode == ColorMode::Default {
        config.color_mode = color_support;
    }

    return config;
}

fn loop_arguments(args: &[&str], config: &mut CliConfig) {
    let mut buffer: Vec<Cow<str>> = vec![];

    for (index, &word) in args.iter().enumerate() {
        if word == "--" {
            let joined = args[(index + 1)..].join(" ");
            buffer.push(Cow::Owned(joined));
            break;
        }

        if word.starts_with("--") {
            let trimmed_switch = &word[2..];
            v_log!(config.logger, "Switch: {}", trimmed_switch);
            resolve_switch(trimmed_switch, config);
        } else if word.starts_with('-') {
            let trimed_switch = &word[1..];
            v_log!(config.logger, "Shorthand group: {}", trimed_switch);
            for char in trimed_switch.chars() {
                resolve_shorthand_switch(char, config);
            }
        } else {
            v_log!(config.logger, "Pushing non-arg: {word}");
            buffer.push(Cow::Borrowed(word));
        }
    }

    config.text_input = buffer.join(" ");
}

fn resolve_shorthand_switch(switch: char, config: &mut CliConfig) {
    match switch {
        'n' => config.newline = false,
        'e' => config.handle_escape = true,
        'c' => config.color_mode = ColorMode::Color256,
        'C' => config.color_mode = ColorMode::NoColor,
        'M' => config.no_markup = true,
        _ => {}
    };
}

fn resolve_switch(word: &str, config: &mut CliConfig) {
    match word {
        "help" => config.extras = ExtraMode::PrintHelp,
        "listc" => config.extras = ExtraMode::ListColors,
        "listcb" => config.extras = ExtraMode::ListColorsAsBackground,
        "lists" => config.extras = ExtraMode::ListStyles,
        "verbose" => config.logger.verbose = true,
        "nobexp" => config.no_binary_expansion = true,
        &_ => {}
    };
}

fn create_logger() -> Logger {
    return Logger {
        verbose: match env::var("VERBOSE_LOG") {
            Ok(_) => true,
            Err(_) => false,
        },
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_parse() {
        let config = parse_args(&["-nc", "Hello,", "--listc", "World!", "--", "!"]);
        assert_eq!(
            config.text_input, "Hello, World! !",
            "Invalid text scanning."
        );
        assert_eq!(config.newline, false, "Newline flag.");
        assert_eq!(config.color_mode, ColorMode::Color256, "Color mode flag.");
    }
}
