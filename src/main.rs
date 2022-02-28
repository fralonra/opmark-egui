mod app;
mod builder;

use clap::Parser as ClapParser;
use opmark::Parser;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

/// Opmark Egui
#[derive(ClapParser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[clap(parse(from_os_str))]
    input: Option<PathBuf>,

    /// Output file
    #[clap(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Build a standalone binary
    #[clap(short, long)]
    standalone: bool,
}

fn build_standalone(input: &Path, output: &Path) {
    let file_content = read_to_string(input).expect("[ERROR] Reading file");

    let parser = Parser::new(file_content);

    let builder = builder::Builder::new(parser);
    let dir = builder.code();
    builder.build(dir.as_path(), output);
}

fn run_app(path: &Path) {
    let file_content = read_to_string(path).expect("[ERROR] Reading file");

    let parser = Parser::new(file_content);

    let app = app::App::new("test", parser);
    let native_options = eframe::NativeOptions {
        always_on_top: false,
        maximized: false,
        decorated: true,
        drag_and_drop_support: false,
        icon_data: None,
        initial_window_size: None,
        resizable: true,
        transparent: false,
    };
    eframe::run_native(Box::new(app), native_options);
}

fn main() {
    let args = Args::parse();

    let input = match &args.input {
        Some(input) => input.as_path(),
        None => Path::new("index.opmark"),
    };
    let output = match &args.output {
        Some(output) => output.as_path(),
        None => Path::new("opmark"),
    };
    if !args.standalone {
        run_app(&input);
    } else {
        build_standalone(&input, &output);
    }
}
