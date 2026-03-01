use rgecko::argparse::parser;
use rgecko::colors::transform;
use rgecko::colors::transform::MarkupOptions;
use rgecko::extras;
use std::env;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    let args = env::args().skip(1);

    let config = parser::parse_args(&args);

    return;

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
            logger: config.logger,
        };

        let processed_text = transform::markup_text(&config.text_input, options);

        print!("{}", processed_text);
    }
}
