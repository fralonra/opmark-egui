mod app;
mod parser;

use clap::Parser as ClapParser;
use parser::Parser;
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
}

fn run_app(path: &Path) {
    let file_content = read_to_string(path).expect("[ERROR] Reading file");
    let root = match path.parent() {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("."),
    };

    let parser = Parser::new(file_content);

    let app = app::App::new(parser.title, root, parser.parser);
    let native_options = eframe::NativeOptions {
        always_on_top: false,
        maximized: if parser.fullscreen { true } else { false },
        decorated: if parser.fullscreen { false } else { true },
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
    run_app(&input);
}
