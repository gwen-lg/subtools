//! A command line utility use subtools functionalities.

use anyhow::Context as _;
use std::{
    borrow::Borrow,
    env,
    path::{Path, PathBuf},
};
use subtools::{FileProcessor, SubtitleFile};

fn main() -> anyhow::Result<()> {
    let paths = if env::args().len() <= 1 {
        let local_path = env::current_dir().context("Failed to access to current directory")?;
        vec![local_path]
    } else {
        env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>()
    };

    paths.into_iter().for_each(|path| {
        if !path.exists() {
            println!("Not a valid path : {path:?}");
        } else if path.is_file() {
            check_file(path);
        } else if path.is_dir() {
            let files_processor = FileProcessor::from_path(path);
            files_processor.subtitle_files().for_each(|file| {
                check_file(file);
            });
        }
    });
    Ok(())
}

fn check_file<F>(file: F)
where
    F: Borrow<Path>,
{
    let file = file.borrow();
    match SubtitleFile::try_from(file) {
        Ok(subfile) => {
            eprintln!("{file:?} is recognized as {}", subfile.format());
        }
        Err(err) => {
            eprintln!("{file:?} is not a recognized subtitle file : {err:?}");
        }
    }
}
