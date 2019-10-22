
use std::{env, process};
use std::path::{PathBuf, Path};
use std::error::Error;

use assembler::run;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() < 2 || args.len() > 3 {
        eprint!("Usage: ");
        let exec_path = Path::new(&args[0]);
        if let Some(exec_name) = exec_path.file_name() {
            eprint!("{}", exec_name.to_string_lossy());
        } else {
            eprint!("{}", exec_path.display());
        }
        eprintln!(" SOURCE OUTPUT");
        process::exit(1);
    }

    let source_file_path = PathBuf::from(&args[1]);

    let output_file_path = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        let mut p = source_file_path.clone();
        p.set_extension("bin");
        p
    };

    if let Err(err) = run(source_file_path, output_file_path) {
        eprintln!("error: {}", err);

        let mut err: &dyn Error = &err;
        while let Some(source) = err.source() {
            eprintln!("reason: {}", source);
            err = source;
        }
    }
}
