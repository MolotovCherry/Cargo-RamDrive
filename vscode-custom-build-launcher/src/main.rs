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

Required:
  DIR                the initial directory used for setting the custom build directory"
    );
}

fn get_hashed_dir(dir: &str) -> String {
    assert!(dir.len() > 0);
    assert!(!dir.ends_with(".."));

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

    let base_name = Path::new(dir)
        .file_name()
        .unwrap()
        .to_str()
        .expect("Failed to convert to string");

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

    if let None = dir.get(0) {
        print_help();
        return;
    }

    let dir = dir[0];

    let hashed_dir = get_hashed_dir(dir);

    let build_dir = target_dir.join(&hashed_dir);
    let build_dir = build_dir.to_str().unwrap();

    Command::new("code.cmd")
        .env("CARGO_BUILD_TARGET_DIR", build_dir)
        .arg(dir)
        .spawn()
        .expect("Failed to launch VsCode");
}
