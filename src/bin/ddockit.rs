use rustgit_wasm_runtime::{analyze_repository, ddockit_publish_endpoint};
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args();
    let _binary = args.next();
    let command = args.next().unwrap_or_default();

    if command != "publish" {
        eprintln!("usage: ddockit publish [repository_path]");
        std::process::exit(1);
    }

    let repository_path = args.next().unwrap_or_else(|| ".".to_string());
    let root = PathBuf::from(repository_path);
    let analysis = match analyze_repository(&root) {
        Ok(analysis) => analysis,
        Err(error) => {
            eprintln!("publish failed: {error}");
            std::process::exit(1);
        }
    };

    let (_, payload) = ddockit_publish_endpoint(&analysis);
    println!("{payload}");
}
