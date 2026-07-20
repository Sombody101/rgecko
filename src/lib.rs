use crate::argparse::parser;
use crate::argparse::parser::CliConfig;
use crate::colors::transform::{MarkupOptions, markup_text};
use std::io::Write;

pub mod argparse;
pub mod colors;
#[macro_use]
pub mod logger;
pub mod extras;

pub fn markup(args: &[&str], out: &mut impl Write) {
    let mut config = parser::parse_args(args);

    let mut prepared_text = String::new();
    if !extras::handle_cli_extras(&config, &mut prepared_text) {
        prepared_text = std::mem::take(&mut config.text_input);
    }

    let final_output = if config.no_markup {
        prepared_text
    } else {
        process_text(&config, prepared_text)
    };

    output_final(&config, final_output, out);
}

fn process_text(config: &CliConfig, text: String) -> String {
    let options = MarkupOptions {
        color_mode: config.color_mode,
        newline: config.newline,
        handle_escape: config.handle_escape,
        no_binary_expansion: config.no_binary_expansion,
        logger: config.logger,
    };

    markup_text(&text, options)
}

fn output_final(config: &CliConfig, mut text: String, out: &mut impl Write) {
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

fn less_forward(text: String) {
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
