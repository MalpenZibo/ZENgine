use std::env;
use std::path::PathBuf;
use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut out_path = manifest_dir.clone();
    
    let target = env::var("TARGET").unwrap();
    if target.contains("pc-windows") {        
        let profile = env::var("PROFILE").unwrap();
        let mut lib_dir = manifest_dir.clone();
        let mut dll_dir = manifest_dir.clone();
        if target.contains("msvc") {
            lib_dir.push("libs/msvc");
            dll_dir.push("libs/msvc");
        }
        else {
            lib_dir.push("libs/gnu-mingw");
            dll_dir.push("libs/gnu-mingw");
        }
        lib_dir.push("lib");
        dll_dir.push("dll");
        if target.contains("x86_64") {
            lib_dir.push("64");
            dll_dir.push("64");
        }
        else {
            lib_dir.push("32");
            dll_dir.push("32");
        }
        println!("cargo:rustc-link-search=all={}", lib_dir.display());
        let sdl_out_path = out_path.clone();
        out_path.push(format!("target/{}", profile));
        for entry in std::fs::read_dir(dll_dir).expect("Can't read DLL dir")  {
            let entry_path = entry.expect("Invalid fs entry").path();
            let file_name_result = entry_path.file_name();
            //new_file_path.push(profile.clone());
            let mut new_file_path = out_path.clone();

            if let Some(file_name) = file_name_result {
                let file_name = file_name.to_str().unwrap();
                if file_name.ends_with(".dll") {
                    new_file_path.push(file_name);
                    std::fs::copy(&entry_path, new_file_path.as_path()).expect("Can't copy from DLL dir");
                }
            }
        }
    }
}