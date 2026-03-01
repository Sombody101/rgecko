#![no_main]
use libfuzzer_sys::fuzz_target;
use rgecko::argparse::parser::ColorMode;
use rgecko::colors::transform;
use rgecko::colors::transform::MarkupOptions;
use rgecko::logger::Logger;

fuzz_target!(|data: &[u8]| {
    if data.len() < 5 {
        return;
    }

    let newline = data[0] % 2 == 0;
    let handle_escape = data[1] % 2 == 0;
    let no_binary_expansion = data[2] % 2 == 0;

    let color_mode = match data[3] % 3 {
        0 => ColorMode::Default,
        1 => ColorMode::Color256,
        _ => ColorMode::NoColor,
    };

    let logger = Logger { verbose: false };

    if let Ok(user_text) = std::str::from_utf8(&data[5..]) {
        let options = MarkupOptions {
            color_mode,
            newline,
            handle_escape,
            no_binary_expansion,
            logger: logger,
        };

        let _ = transform::markup_text(user_text, options);
    }
});
