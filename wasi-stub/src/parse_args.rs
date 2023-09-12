use crate::Error;
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    path::PathBuf,
};

pub(crate) struct Args {
    pub binary: Vec<u8>,
    pub path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub list: bool,
}

enum Arg {
    Plain {
        name: &'static str,
        required: bool,
        help: &'static str,
    },
    LongFlag {
        name: &'static str,
        help: &'static str,
    },
    #[allow(dead_code)]
    ShortFlag { flag: char, help: &'static str },
    KeyValue {
        keys: &'static [&'static str],
        value_type: &'static str,
        help: &'static str,
    },
}

struct TestArgParser {
    command_name: &'static str,
    help: &'static str,
    args: Vec<Arg>,
    plain_args: HashMap<String, OsString>,
    long_flags: HashSet<String>,
    short_flags: HashSet<char>,
    key_values: HashMap<String, OsString>,
    requested_help: bool,
}
impl TestArgParser {
    fn new(command_name: &'static str, help: &'static str, args: Vec<Arg>) -> Self {
        Self {
            command_name,
            help,
            args,
            plain_args: Default::default(),
            long_flags: Default::default(),
            short_flags: Default::default(),
            key_values: Default::default(),
            requested_help: false,
        }
    }

    fn parse(&mut self) -> Result<(), String> {
        let mut expect_long_flags = HashSet::new();
        let mut expect_short_flags = HashSet::new();
        let mut expect_keys = HashSet::new();
        let mut remaining_plain_args: Vec<_> = self
            .args
            .iter()
            .filter_map(|a| {
                match a {
                    Arg::Plain { name, required, .. } => return Some((*name, *required)),
                    Arg::LongFlag { name, .. } => {
                        expect_long_flags.insert(*name);
                    }
                    Arg::ShortFlag { flag, .. } => {
                        expect_short_flags.insert(*flag);
                    }
                    Arg::KeyValue { keys, .. } => {
                        for key in *keys {
                            expect_keys.insert(*key);
                        }
                    }
                }
                None
            })
            .collect();

        let mut current_key = None;
        for arg in std::env::args_os().skip(1) {
            if let Some(key) = current_key.take() {
                self.key_values.insert(key, arg);
                continue;
            }
            let arg = match arg.to_str() {
                Some(a) => a,
                None => {
                    if let Some(plain) = remaining_plain_args.pop() {
                        self.plain_args.insert(plain.0.to_owned(), arg);
                        continue;
                    } else {
                        return Err(format!("Unexpected argument: '{arg:?}'"));
                    }
                }
            };
            if arg == "--help" || arg == "-h" {
                self.requested_help = true;
                continue;
            }
            if expect_long_flags.contains(arg) {
                self.long_flags.insert(arg.to_owned());
            } else if expect_keys.contains(arg) {
                current_key = Some(arg.to_owned());
            } else if arg.starts_with('-') {
                for c in arg.chars().skip(1) {
                    if expect_short_flags.contains(&c) {
                        self.short_flags.insert(c);
                    } else {
                        return Err(format!("Unknown short option: {c}"));
                    }
                }
            } else if let Some(plain) = remaining_plain_args.pop() {
                self.plain_args.insert(plain.0.to_owned(), arg.into());
            } else {
                return Err(format!("Unexpected argument: '{arg}'"));
            }
        }
        if !self.requested_help {
            for (plain_arg, required) in remaining_plain_args {
                if required {
                    return Err(format!("Missing argument {plain_arg}"));
                }
            }
        }
        Ok(())
    }

    fn print_help_message(&self) {
        println!("{} {}", self.command_name, env!("CARGO_PKG_VERSION"));
        println!();
        println!("{}", self.help);
        println!();
        println!("USAGE:");
        print!("    {} [OPTIONS]", self.command_name);
        for (name, required) in self.args.iter().filter_map(|a| match a {
            Arg::Plain { name, required, .. } => Some((*name, *required)),
            _ => None,
        }) {
            if required {
                print!(" <{}>", name)
            } else {
                print!(" [{}]", name)
            }
        }
        println!();
        println!();
        for arg in &self.args {
            match arg {
                Arg::Plain {
                    name,
                    required,
                    help,
                } => {
                    if *required {
                        println!("    <{name}>");
                    } else {
                        println!("    [{name}]");
                    }
                    println!("        {help}");
                }
                _ => continue,
            }
        }
        println!();
        println!("OPTIONS:");
        for arg in &self.args {
            match arg {
                Arg::Plain { .. } => continue,
                Arg::LongFlag { name, help } => {
                    println!("    {name}");
                    println!("        {help}");
                }
                Arg::ShortFlag { flag, help } => {
                    println!("    -{flag}");
                    println!("        {help}");
                }
                Arg::KeyValue {
                    keys,
                    value_type,
                    help,
                } => {
                    print!("    ");
                    for (i, key) in keys.iter().enumerate() {
                        if i != 0 {
                            print!(", ");
                        }
                        print!("{key}")
                    }
                    println!(" <{value_type}>");
                    println!("        {help}");
                }
            }
        }
    }
}

impl Args {
    pub fn new() -> Result<Self, Error> {
        let mut arg_parser = TestArgParser::new(
            env!("CARGO_PKG_NAME"),
            "A command to replace wasi functions with stubs. The stubbed function can still be called, but they won't have any side-effect, and will simply return dummy values.",
            vec![
                Arg::Plain {
                    name: "file",
                    required: true,
                    help: "Input wasm file.",
                },
                Arg::KeyValue {
                    keys: &["-o", "--output"],
                    value_type: "PATH",
                    help: "Specify the output path.",
                },
                Arg::LongFlag {
                    name: "--list",
                    help: "List the functions to stub, but don't write anything.",
                },
            ],
        );

        arg_parser.parse()?;

        if arg_parser.requested_help {
            arg_parser.print_help_message();
            std::process::exit(0);
        }

        let path = PathBuf::from(&arg_parser.plain_args["file"]);
        let list = arg_parser.long_flags.contains("--list");
        let mut output_path = None;

        if let Some(path) = arg_parser.key_values.get("--output") {
            output_path = Some(PathBuf::from(path));
        }
        Ok(Self {
            binary: std::fs::read(&path)?,
            path,
            output_path,
            list,
        })
    }
}
