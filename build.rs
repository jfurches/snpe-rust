#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code
)]

extern crate bindgen;
extern crate reqwest;
extern crate unzip;
use std::env;
use std::path::PathBuf;
use unzip::Unzipper;

fn main() {
    let sdk_dir = setup_snapdragon_sdk();
    linklibs(&sdk_dir);
    generate_bindings(&sdk_dir.join("include"));
}

/// Downloads and unzips the Qualcomm Snapdragon SDK, returning the path to the sdk
fn setup_snapdragon_sdk() -> PathBuf {
    let url = "https://softwarecenter.qualcomm.com/api/download/software/qualcomm_neural_processing_sdk/v2.26.0.240828.zip";

    // Download the qualcomm sdk to the current directory as "snapdragon_sdk.zip"
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let download_path = PathBuf::from(project_dir.clone()).join("snapdragon_sdk.zip");
    let unzipped_path = PathBuf::from(project_dir.clone()).join("snapdragon_sdk");

    if !unzipped_path.exists() {
        // Download the zip file if necessary
        if !download_path.exists() {
            print!("Downloading snapdragon sdk...");
            let mut zipfile = std::fs::File::create(&download_path).unwrap();
            let mut response = reqwest::blocking::get(url).unwrap();
            response
                .copy_to(&mut zipfile)
                .expect("Failed to download snapdragon sdk");
        }

        // Unzip the downloaded file
        println!("Extracting snapdragon sdk...");
        let zipfile = std::fs::File::open(&download_path).unwrap();
        let unzipper = Unzipper::new(zipfile, &unzipped_path);
        unzipper.unzip().expect("Failed to unzip snapdragon sdk");

        // Delete the zip file
        std::fs::remove_file(&download_path).unwrap();
    }

    // Walk the directory and find the include and lib directories. The structure looks like qairt/<version>/. We need the version number
    let sdk_dir = unzipped_path.join("qairt");
    let entry = sdk_dir
        .read_dir()
        .unwrap()
        .next()
        .expect("Empty sdk directory");
    let entry = entry.unwrap();
    let version = entry
        .file_name()
        .to_str()
        .expect("Failed to read version number")
        .to_string();

    println!("Using Qualcomm Snapdragon SDK v{}", version);
    let sdk_dir = sdk_dir.join(version);
    sdk_dir
}

fn linklibs(sdk_dir: &PathBuf) {
    let target = env::var("TARGET").unwrap();
    let lib_dir = match target.as_str() {
        // Windows on x86 and arm
        "x86_64-pc-windows-msvc" => "x86_64-windows-msvc",
        "aarch64-pc-windows-msvc" => "arm64x-windows-msvc",

        // Android
        "aarch64-linux-android" => "aarch64-android",

        // Linux
        "x86_64-unknown-linux-gnu" => "x86_64-linux-clang",
        "aarch64-unknown-linux-gnu" => "aarch64-ubuntu-gcc9.4",

        _ => panic!("Unsupported platform: {}", target),
    };

    let native_lib_dir = sdk_dir.join("lib").join(lib_dir);

    println!(
        "cargo:rustc-link-search=native={}",
        native_lib_dir.to_str().unwrap()
    );

    for lib in native_lib_dir.read_dir().unwrap() {
        let lib = lib.unwrap();

        if let Some(ext) = lib.path().extension() {
            if ext == "so" || ext == "dll" {
                println!(
                    "cargo:rustc-link-lib=dylib={}",
                    lib.file_name().to_str().unwrap()
                );
            } else if ext == "a" || ext == "lib" {
                println!(
                    "cargo:rustc-link-lib=static={}",
                    lib.file_name().to_str().unwrap()
                );
            }
        }
    }
}

/// Generates bindings to the C api
fn generate_bindings(include_dir: &PathBuf) {
    use std::collections::HashMap;

    println!("Using include dir: {}", include_dir.to_str().unwrap());
    let mut include_paths = HashMap::new();
    include_paths.insert("QNN", "QnnBackend.h");
    include_paths.insert("Genie", "GenieDialog.h");

    for (prefix, header) in include_paths {
        let header_include_dir = include_dir.join(prefix);
        let header_file = header_include_dir.join(header);
        assert!(header_include_dir.join(header).exists());
        let include = header_include_dir.to_str().unwrap();
        let include_arg = format!("--include-directory={}/", include);
        println!("{}", include_arg);
        let bindings = bindgen::Builder::default()
            .clang_arg(include_arg)
            .header(header_file.to_str().unwrap())
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join(format!("{}_bindings.rs", prefix.to_lowercase())))
            .expect(format!("Couldn't write bindings for {}!", prefix).as_str());
    }
}
