use crate::Error;
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

pub(crate) struct Args {
    pub binary: Vec<u8>,
    pub path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub list: bool,
}

impl Args {
    pub fn new(args: impl Iterator<Item = OsString>) -> Result<Self, Error> {
        let mut path = None;
        let mut list = false;
        let mut output_path = None;

        let mut prev_arg = OsString::new();
        for arg in args {
            let is_known_flag = matches!(
                arg.as_os_str().to_str().unwrap_or(""),
                "--list" | "-o" | "--output"
            );
            let is_flag = arg
                .as_os_str()
                .to_str()
                .map(|s| s.starts_with('-'))
                .unwrap_or(false);

            if arg.as_os_str() == OsStr::new("--list") {
                list = true;
            } else if prev_arg.as_os_str() == OsStr::new("-o")
                || prev_arg.as_os_str() == OsStr::new("--output")
            {
                output_path = Some(PathBuf::from(&arg));
            } else if !is_known_flag {
                if is_flag {
                    return Err(format!("unknown flag: {arg:?}").into());
                } else if path.is_some() {
                    return Err(format!("unexpected argument: {arg:?}").into());
                } else {
                    path = Some(PathBuf::from(&arg));
                }
            }
            prev_arg = arg;
        }
        match (path, list) {
            (None, false) => Self::print_help_and_exit(),
            (None, true) => Err("".into()),
            (Some(path), _) => Ok(Self {
                binary: std::fs::read(&path)?,
                path,
                output_path,
                list,
            }),
        }
    }

    fn print_help_and_exit() -> ! {
        println!("Usage: wasi-stub file.wasm");
        std::process::exit(0);
    }
}
