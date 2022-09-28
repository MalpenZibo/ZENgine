use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
use std::error::Error;
use std::path::PathBuf;

fn add_lib(_name: impl AsRef<str>, _static: bool) {
    println!(
        "cargo:rustc-link-lib={}{}",
        if _static { "static=" } else { "" },
        _name.as_ref()
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "android" {
        add_lib("c++_shared", false);
    }

    if env::var("CARGO_CFG_TARGET_OS").unwrap() != "android"
        && env::var("CARGO_CFG_TARGET_FAMILY").unwrap() != "wasm"
    {
        // This tells cargo to rerun this script if something in /assets/ changes.
        println!("cargo:rerun-if-changed=assets/*");

        let paths_to_copy = vec!["assets/"];
        let mut out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

        // destination_assets_dir is something like {workspace}/target/{profile}/build/pong-{hash}/out
        out_dir.pop();
        out_dir.pop();
        out_dir.pop();

        let mut copy_options = CopyOptions::new();
        copy_options.overwrite = true;
        copy_items(&paths_to_copy, out_dir, &copy_options)?;
    }

    Ok(())
}
