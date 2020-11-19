use structopt::StructOpt;
use log::LevelFilter;

use shader_processor::*;

fn main() {
    let args = ShaderProcessorArgs::from_args();

    // Setup logging
    let level = if args.trace {
        LevelFilter::Trace
    } else {
        LevelFilter::Error
    };

    env_logger::Builder::from_default_env()
        .default_format_timestamp_nanos(true)
        .filter_level(level)
        .init();

    if let Err(e) = run(&args) {
        eprintln!("{}", e.to_string());
    }
}