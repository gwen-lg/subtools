//! A command line utility use subtools functionalities.

use std::{env, path::PathBuf};

use subtools::SubtitleFile;

fn main() -> anyhow::Result<()> {
    env::args().skip(1).for_each(|arg| {
        let path = PathBuf::from(arg);
        if !path.exists() {
            println!("Not a valid path : {path:?}");
        } else if !path.is_file() {
            println!("Not a file : {path:?}");
        } else {
            match SubtitleFile::try_from(path.as_path()) {
                Ok(subfile) => {
                    println!("{path:?} is a {}", subfile.format());
                }
                Err(err) => {
                    println!("Arg {path:?} is not recognized as subtitle file : {err}");
                }
            }
        }
    });
    Ok(())
}
