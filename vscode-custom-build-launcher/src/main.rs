// Hide the console window
#![windows_subsystem = "windows"]

#[cfg(not(target_os = "windows"))]
compile_error!("This is unsupported on your OS");

use std::env;
use std::path::Path;

use getargs::{Opt, Options};
use smallvec::SmallVec;

use std::process::Command;

fn print_help() {
    println!(
        r"Usage: vscode-custom-build-launcher [OPTIONS] <DIR>

This will launch VsCode with the CARGO_BUILD_TARGET_DIR env var set to the input directory.
By default, it will use TMP to calculate the path to the custom build directory, but you can
change this below with a flag

Options:
  -h, --help         print this help
  -t, --target <VAL> custom target build directory

Positional:
  DIR                the initial directory used for setting the custom build directory.
                     if this is omitted, it will start vscode without any specific directory"
    );
}

fn get_hashed_dir(dir: &str) -> String {
    assert!(dir.len() > 0);
    assert!(!dir.ends_with(".."));

    let path = Path::new(dir);

    let base_name;

    // Properly handle a drive letter vs regular path
    let components = path.components().collect::<Vec<_>>();
    if components.len() <= 2 {
        let drive = components[0]
            .as_os_str()
            .to_str()
            .expect("Unable to convert to string");

        base_name = drive.replace(":", "");
    } else {
        base_name = Path::new(dir)
            .file_name()
            .expect("Not a valid path")
            .to_str()
            .expect("Failed to convert to string")
            .to_string();
    }

    // Initial prime and offset chosen for 32-bit output
    // See https://en.wikipedia.org/wiki/Fowler–Noll–Vo_hash_function
    const FNV_PRIME: u64 = 16777619;
    const OFFSET: u64 = 2166136261;
    const POWER: u64 = u64::pow(2, 32);

    let bytes = dir.as_bytes();

    // Copy offset as initial hash value
    let mut hash = OFFSET;

    for octet in bytes {
        hash ^= *octet as u64;
        hash = hash * FNV_PRIME % POWER;
    }

    format!("{base_name}-{hash}")
}

fn main() {
    let args = env::args().skip(1).collect::<SmallVec<[_; 4]>>();
    let args_iter = args.iter().map(|i| &**i);

    let mut opts = Options::new(args_iter);

    // the base directory we will use. we will be creating a custom build directory inside this one
    let mut target_dir = Path::new(&env::var("TMP").expect("Missing the TMP env var")).join("rust");

    while let Some(opt) = opts.next_opt().expect("argument parsing error") {
        match opt {
            Opt::Short('h') | Opt::Long("help") => {
                print_help();
                return;
            }

            Opt::Short('t') | Opt::Long("target") => {
                let val = opts.value_opt();

                if let None = val {
                    eprint!("Target requires a value");
                    return;
                }

                target_dir = Path::new(&val.unwrap().to_string()).to_path_buf();
            }

            _ => (),
        }
    }

    // the rust source code directory, which we use to calculate the value to append to the base directory
    let dir = opts.positionals().take(1).collect::<SmallVec<[_; 1]>>();
    let dir = dir.get(0);

    let mut command = Command::new("code");

    if let Some(dir) = dir {
        let hashed_dir = get_hashed_dir(dir);

        let build_dir = target_dir.join(&hashed_dir);
        let build_dir = build_dir.to_str().unwrap();

        command.env("CARGO_BUILD_TARGET_DIR", build_dir).arg(dir);
    }

    command.spawn().expect("Failed to launch VsCode");
}
