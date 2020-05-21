use std::env;
use std::fs::{self};
use std::path::Path;
use std::path::PathBuf;

fn main() {
    let mut source_assets_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut destination_assets_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    source_assets_dir.push("assets");

    // something like {workspace}/target/{profile}/build/pong-{hash}/out
    destination_assets_dir.pop();
    destination_assets_dir.pop();
    destination_assets_dir.pop();
    destination_assets_dir.push("assets");

    copy_dir_content_recursive(source_assets_dir.as_path(), &destination_assets_dir);
}

// one possible implementation of walking a directory only visiting files
fn copy_dir_content_recursive(dir: &Path, destination_dir: &PathBuf) {
    if dir.is_dir() {
        if !destination_dir.as_path().exists() {
            fs::create_dir(destination_dir.as_path()).unwrap_or_else(|_| {
                panic!(
                    "Can't create destination folder {:?}",
                    destination_dir.as_path()
                )
            });
        }
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                let mut new_destination = destination_dir.clone();
                new_destination.push(entry.file_name());
                copy_dir_content_recursive(&path, &new_destination);
            } else {
                let mut new_file_path = destination_dir.clone();
                new_file_path.push(entry.file_name());
                match std::fs::copy(&path, new_file_path.as_path()) {
                    Ok(_v) => {}
                    Err(e) => panic!(
                        "Can't copy from Resource dir: {} - from: {:?} to: {:?}",
                        e, path, new_file_path
                    ),
                };
            }
        }
    }
}
