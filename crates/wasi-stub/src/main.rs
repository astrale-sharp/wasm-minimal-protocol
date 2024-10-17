mod parse_args;

use std::path::PathBuf;
use wasi_stub::stub_wasi_functions;

// Error handling
struct Error(Box<dyn std::fmt::Display>);
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
impl<E: std::fmt::Display + 'static> From<E> for Error {
    fn from(err: E) -> Self {
        Self(Box::new(err))
    }
}

fn main() -> Result<(), Error> {
    let parse_args::Args {
        binary,
        path,
        output_path,
        list,
        should_stub,
        return_value,
    } = parse_args::Args::new()?;

    let output = stub_wasi_functions(&binary, should_stub, return_value)?;

    if !list {
        write_output(path, output_path, output)?;
    } else {
        println!("NOTE: no output produced because the '--list' option was specified")
    }

    Ok(())
}

fn write_output(path: PathBuf, output_path: Option<PathBuf>, output: Vec<u8>) -> Result<(), Error> {
    let output_path = match output_path {
        Some(p) => p,
        // Try to find an unused output path
        None => {
            let mut i = 0;
            let mut file_name = path.file_stem().unwrap().to_owned();
            file_name.push(" - stubbed.wasm");
            loop {
                let mut new_path = path.clone();
                if i > 0 {
                    let mut file_name = path.file_stem().unwrap().to_owned();
                    file_name.push(format!(" - stubbed ({i}).wasm"));
                    new_path.set_file_name(&file_name);
                } else {
                    new_path.set_file_name(&file_name);
                }
                if !new_path.exists() {
                    break new_path;
                }
                i += 1;
            }
        }
    };
    std::fs::write(&output_path, output)?;
    let permissions = std::fs::File::open(path)?.metadata()?.permissions();
    std::fs::File::open(output_path)?.set_permissions(permissions)?;
    Ok(())
}
