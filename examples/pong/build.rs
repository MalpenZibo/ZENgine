use std::env;
use std::fs::{self};
use std::path::Path;
use std::path::PathBuf;

fn add_lib(_name: impl AsRef<str>, _static: bool) {
    #[cfg(not(feature = "test"))]
    println!(
        "cargo:rustc-link-lib={}{}",
        if _static { "static=" } else { "" },
        _name.as_ref()
    );
}

fn main() {
    add_lib("c++_shared", false);

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
fn copy_dir_content_recursive(dir: &Path, destination_dir: &Path) {
    if dir.is_dir() {
        if !destination_dir.exists() {
            fs::create_dir(destination_dir).unwrap_or_else(|_| {
                panic!("Can't create destination folder {:?}", destination_dir)
            });
        }
        for entry in fs::read_dir(dir).unwrap() {
            let unwrapped_entry = entry.unwrap();
            let path = unwrapped_entry.path();
            let mut new_destination = destination_dir.to_path_buf();
            new_destination.push(unwrapped_entry.file_name());
            if path.is_dir() {
                copy_dir_content_recursive(&path, &new_destination);
            } else {
                let mut new_file_path = destination_dir.to_path_buf();
                new_file_path.push(unwrapped_entry.file_name());
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
