use crate::argparse::parser;
use crate::argparse::parser::CliConfig;
use crate::colors::ztran::{MarkupOptions, markup_text};
use std::io::Write;

pub mod argparse;
pub mod colors;
#[macro_use]
pub mod logger;
pub mod extras;

pub fn markup_write(args: &[&str], out: &mut impl Write) {
    let config = parser::parse_args(args);
    markup_write_cli(&config, out);
}

pub fn markup_write_cli(config: &CliConfig, out: &mut impl Write) {
    cli_extras(&config);

    let final_output = if config.no_markup {
        config.text_input.as_str()
    } else {
        &process_text(&config)
    };

    output_final(&config, final_output, out);
}

pub fn markup_args(args: &[&str]) -> String {
    let mut config = parser::parse_args(args);

    cli_extras(&mut config);

    if config.no_markup {
        config.text_input
    } else {
        process_text(&config)
    }
}

pub fn markup_string(text: &str, config: &MarkupOptions) -> String {
    markup_text(text, config)
}

fn cli_extras(config: &CliConfig) {
    let mut prepared_text = String::new();
    if !extras::handle_cli_extras(&config, &mut prepared_text) {
        //prepared_text = std::mem::take(&mut config.text_input);
    }
}

fn process_text(config: &CliConfig) -> String {
    let options = MarkupOptions {
        color_mode: config.color_mode,
        handle_escape: config.handle_escape,
        no_binary_expansion: config.no_binary_expansion,
        logger: config.logger,
    };

    //colors::transform::markup_text(&config.text_input, options)
    markup_text(&config.text_input, &options)
}

fn output_final(config: &CliConfig, text: &str, out: &mut impl Write) {
    if config.interactive {
        v_log!(config.logger, "Forwarding to less");
        less_forward(text);
        return;
    }

    if config.newline {
        _ = write!(out, "{text}\n");
    } else {
        _ = write!(out, "{text}");
    }
}

fn less_forward(text: &str) {
    if let Ok(mut child) = std::process::Command::new("less")
        .args(["-RSXF"])
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = Write::write_all(&mut stdin, text.as_bytes());
        }

        let _ = child.wait();

        return;
    }
}
