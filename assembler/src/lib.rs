
use std::path::{Path, PathBuf};
use std::{error, fmt, io};
use std::fs::File;
use std::io::{BufReader, BufRead};

#[derive(Debug)]
pub enum AssemblerError {
    FileOpen(PathBuf, io::Error),
    FileRead(PathBuf, io::Error),
}

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            AssemblerError::FileOpen(path, _) => write!(f, "unable to open {}", path.display()),
            AssemblerError::FileRead(path, _) => write!(f, "failed to read {}", path.display()),
        }
    }
}

impl error::Error for AssemblerError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            AssemblerError::FileOpen(_, io_error) => Some(io_error),
            AssemblerError::FileRead(_, io_error) => Some(io_error),
        }
    }
}

pub fn run(source_path: PathBuf, output_path: PathBuf) -> Result<(), AssemblerError> {
    let source_file = File::open(&source_path)
        .map_err(|err| AssemblerError::FileOpen(source_path.clone(), err))?;

    let source_reader = BufReader::new(source_file);

    for line in source_reader.lines() {
        let line = line
            .map_err(|err| AssemblerError::FileRead(source_path.clone(), err))?;

        println!("{}", line);
    }

    Ok(())
}
