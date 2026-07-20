use std::env;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let s_args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();

    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    rgecko::markup(&s_args, &mut std::io::stdout());
}
