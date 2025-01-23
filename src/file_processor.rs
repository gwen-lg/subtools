use std::{
    fs::{self, File, ReadDir},
    path::{Path, PathBuf},
};

//TODO: manage errors
/// An helper to manage file processing (iterating, filtering, ...).
pub struct FileProcessor {
    //attr: Metadata,
    path: PathBuf,
}

impl FileProcessor {
    //TODO: replace panic with Error ?
    /// Create a [`FileProcessor`] from a path.
    /// # Panics
    ///
    /// Will panic if `fs::canonicalize` return an Error. TODO: replace by an error.
    #[must_use]
    pub fn from_path(path: PathBuf) -> Self {
        //TODO: keep file opened
        let path = fs::canonicalize(path).unwrap();
        //let attr = fs::metadata(path).unwrap();
        Self { path }
    }

    //TODO manage format wanted (text, images, srt, vobsub, psg, ...)
    /// Create an iterator on subtitles files
    #[must_use]
    pub fn subtitle_files(&self) -> SubFiles {
        SubFiles::new(self.path.as_path())
    }
}

enum State {
    File(PathBuf),
    Directory(ReadDir),
}

pub struct SubFiles {
    state: Option<State>,
}

impl SubFiles {
    fn new(path: &Path) -> Self {
        let state = match fs::read_dir(path) {
            Ok(read_dir) => Some(State::Directory(read_dir)),
            Err(err) => {
                if path.is_file() {
                    Some(State::File(path.into()))
                } else {
                    eprintln!("can't read {path:?} :\n\t{err}");
                    todo!();
                }
            }
        };
        Self { state }
    }
}

impl Iterator for SubFiles {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(state) = self.state.take() {
            match state {
                State::Directory(mut dir) => {
                    let next = next_file(&mut dir);
                    if next.is_some() {
                        self.state = Some(State::Directory(dir)); //Keep file
                    }
                    next
                }
                State::File(sub_file) => Some(sub_file),
            }
        } else {
            None
        }
    }
}

//TODO: rework with better management of the loop
// better handle the ReadDir ?
fn next_file(dir: &mut ReadDir) -> Option<PathBuf> {
    for entry in dir {
        match entry {
            Ok(entry) => {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    return Some(entry_path);
                } else if entry_path.is_dir() {
                    // ignore director if don't manage recursive
                } else {
                    todo!()
                }
            }
            Err(err) => {
                eprintln!("Can't read {err:?}");
                //simply log error ?
                //need a context to send log
            }
        }
    }
    None
}

///TODO
pub fn filter_text_subs<P>(path: P) -> Option<File>
where
    P: AsRef<Path>,
{
    if let Some(file_ext) = path.as_ref().extension() {
        if file_ext == "srt" {
            File::open(path)
                .inspect_err(|err| eprintln!("filter_text_subs : {err:?}"))
                .ok()
        } else {
            //TODO manage other extension
            None
        }
    } else {
        //TODO: mange no extension ?
        None
    }
}
