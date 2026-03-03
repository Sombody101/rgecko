use rgecko::argparse::parser;
use rgecko::argparse::parser::CliConfig;
use rgecko::colors::transform::{MarkupOptions, markup_text};
use rgecko::{extras, v_log};
use std::env;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let args = env::args().skip(1);

    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

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

    output_final(&config, final_output);
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

fn output_final(config: &CliConfig, mut text: String) {
    if config.newline {
        text.push('\n');
    }

    if config.interactive {
        v_log!(config.logger, "Forwarding to less");
        less_forward(text);
        return;
    }

    print!("{}", text);
}

fn less_forward(text: String) {
    if let Ok(mut child) = std::process::Command::new("less")
        .args(["-R", "-S", "-X", "-F"])
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = std::io::Write::write_all(&mut stdin, text.as_bytes());
        }

        let _ = child.wait();

        return;
    }
}
