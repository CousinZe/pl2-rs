use std::env::current_dir;
use std::path::PathBuf;

fn main() {
    let source_path: PathBuf = current_dir()
        .unwrap()
        .join("pl2")
        .join("pl2b.c");
    cc::Build::new()
        .file(source_path)
        .compile("pl2");
}
