use rgecko::argparse::parser;
use rgecko::colors::transform;
use rgecko::colors::transform::MarkupOptions;
use rgecko::extras;
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let args_slices: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let config = parser::parse_args(&args_slices);

    if extras::handle_cli_extras(&config) {
        return;
    }

    if config.no_markup {
        print!("{}", config.text_input);
    } else {
        let options = MarkupOptions {
            color_mode: config.color_mode,
            newline: config.newline,
            handle_escape: config.handle_escape,
            no_binary_expansion: config.no_binary_expansion,
            logger,
        };

        let processed_text = transform::markup_text(&config.text_input, options);

        print!("{}", processed_text);
    }
}
