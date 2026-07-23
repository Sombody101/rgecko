use std::env;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let s_args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
    let config = rgecko::argparse::parser::parse_args(&s_args);

    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    rgecko::markup_write_cli(&config, &mut std::io::stdout());
}
