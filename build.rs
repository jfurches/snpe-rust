#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code
)]

extern crate bindgen;
extern crate reqwest;
extern crate unzip;
use bindgen::BindgenError;
use std::collections::HashMap;
use std::fs::{DirEntry, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::{env, fs};
use unzip::Unzipper;

fn main() {
    // let sdk_dir = setup_snapdragon_sdk();
    let sdk_dir = PathBuf::from(std::env::var("SNPE_ROOT").unwrap_or("/snpe".to_string()));
    setup_env_variables(&sdk_dir).expect("Failed to set environment variables");
    let platform_dir = linklibs(&sdk_dir);

    let include_dir = sdk_dir.join("include");
    generate_genie_bindings(&include_dir);
    generate_qnnbackend_bindings(&include_dir);
    generate_snpe_bindings(&include_dir).expect("Failed to bind SNPE library");
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

fn setup_env_variables(sdk_dir: &PathBuf) -> Result<(), io::Error> {
    let output = std::process::Command::new("./printenv.sh")
        .arg(sdk_dir.to_str().unwrap())
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to run printenv.sh: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    // Capture the output
    let output_str = String::from_utf8_lossy(&output.stdout);
    let desired_variables = vec!["PYTHONPATH", "LD_LIBRARY_PATH", "SNPE_ROOT", "PATH"];

    // Parse and set environment variables
    for line in output_str.lines() {
        if let Some((key, value)) = line.split_once('=') {
            if desired_variables.contains(&key) {
                env::set_var(key, value);
                println!("cargo:rustc-env={}={}", key, value);
            }
        }
    }

    Ok(())
}

fn linklibs(sdk_dir: &PathBuf) -> &str {
    let target = env::var("TARGET").unwrap();
    let platform_dir = match target.as_str() {
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

    let native_lib_dir = sdk_dir.join("lib").join(platform_dir);
    println!("cargo:rustc-env=SNPE_LIB_DIR={}", native_lib_dir.display());

    println!(
        "cargo:rustc-link-search=all={}",
        native_lib_dir.to_str().unwrap()
    );

    for lib in native_lib_dir.read_dir().unwrap() {
        let lib = lib.unwrap();

        if let Some(ext) = lib.path().extension() {
            let name = library_name(&lib);
            if ext == "so" || ext == "dll" {
                println!("cargo:rustc-link-lib=dylib={}", name);
            } else if ext == "a" || ext == "lib" {
                println!("cargo:rustc-link-lib=static={}", name);
            }
        }
    }

    platform_dir
}

/// Takes a path like /path/to/libLibrary.so and returns the library name (Library)
fn library_name(entry: &DirEntry) -> String {
    let path = entry.path();
    let name = path.file_stem().unwrap().to_str().unwrap();
    String::from(&name[3..])
}

/// Generates bindings to the C api
fn generate_genie_bindings(include_dir: &PathBuf) {
    let header_include_dir = include_dir.join("Genie");
    let header_file = header_include_dir.join("GenieDialog.h");
    assert!(header_file.exists());

    let include = header_include_dir.to_str().unwrap();
    let include_arg = format!("--include-directory={}/", include);
    println!("{}", include_arg);
    let bindings = bindgen::Builder::default()
        .clang_arg(include_arg)
        .header(header_file.to_str().unwrap())
        .raw_line("#[allow(warnings)]")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("genie_bindings.rs"))
        .expect("Couldn't write bindings for genie!");
}

fn generate_qnnbackend_bindings(include_dir: &PathBuf) {
    let mut devices: HashMap<&str, &str> = HashMap::new();

    devices.insert("cpu", "QnnCpu");
    devices.insert("gpu", "QnnGpu");
    devices.insert("htp", "QnnHtp");

    for (device, lib) in devices {
        let header_include_dir = include_dir.join("QNN");
        let header_file = header_include_dir.join("QnnBackend.h");
        assert!(header_file.exists());

        let include = header_include_dir.to_str().unwrap();
        let include_arg = format!("--include-directory={}/", include);
        println!("{}", include_arg);
        let bindings = bindgen::Builder::default()
            .clang_arg(include_arg)
            .header(header_file.to_str().unwrap())
            .raw_line("#[allow(warnings, non_camel_case_types, non_snake_case)]")
            .dynamic_library_name(lib)
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join(format!("qnn{}_bindings.rs", device)))
            .expect(format!("Couldn't write bindings for {}!", device).as_str());
    }
}

fn generate_snpe_bindings(include_dir: &PathBuf) -> Result<(), BindgenError> {
    let header_include_dir = include_dir.join("SNPE");
    let snpe_dir = header_include_dir.join("SNPE");

    let include = header_include_dir.to_str().unwrap();
    let include_arg = format!("--include-directory={}/", include);
    println!("{}", include_arg);
    let builder = bindgen::Builder::default()
        .clang_arg(include_arg)
        .header(snpe_dir.join("SNPE.h").to_str().unwrap())
        .header(snpe_dir.join("SNPEUtil.h").to_str().unwrap())
        .header(
            header_include_dir
                .join("DlContainer")
                .join("DlContainer.h")
                .to_str()
                .unwrap(),
        )
        .dynamic_library_name("SNPE")
        .raw_line("#[allow(warnings, non_camel_case_types, non_snake_case)]");

    let mut result = builder.clone().generate();
    // getting an error here is likely because DIEnums.h uses bool types without
    // importing <stdbool.h>. So we'll add that to the file then retry.
    if result.is_err() {
        patch_header_file(include_dir).expect("Couldn't patch header file");
        result = builder.generate();
    }

    let bindings = result?;
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("snpe_bindings.rs"))
        .expect("Couldn't write bindings for snpe!");

    Ok(())
}

fn patch_header_file(include_dir: &PathBuf) -> Result<(), std::io::Error> {
    let header_file = include_dir.join("SNPE").join("DlSystem").join("DlEnums.h");
    let tmp_file = header_file.with_extension("tmp");

    let file = OpenOptions::new().read(true).open(header_file.clone())?;
    let reader = BufReader::new(file);

    let temp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(tmp_file.clone())?;
    let mut writer = BufWriter::new(temp_file);

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 22 {
            // Insert at line 23 (0-indexed)
            writeln!(writer, "#include <stdbool.h>")?;
        }
        writeln!(writer, "{}", line)?;
    }

    fs::rename(tmp_file, header_file)?;
    Ok(())
}
