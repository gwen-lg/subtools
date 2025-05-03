//! `subtools` is a library line app to check and manipulate subtitles.
//! A command line utilities is also provided.

mod app_context;
mod check;
mod extract;
mod file_encoding;
mod file_processor;
pub mod matroska; //TODO: move in a crate (subtile ? or subtile-matroska ?)
mod ocr;
mod regex;
mod spellcheck;
mod subtitle_file;

pub use app_context::{
    AppContext, IterProcessing, ProcErrLogger, ProcessingContext, ProcessingContextIter,
    ProcessingProgress, SubProcess,
};
pub use check::check_subs;
pub use extract::extract_subs;
pub use file_encoding::convert_subs_to_utf8;
pub use file_processor::FileProcessor;
pub use ocr::ocr_subs;
pub use spellcheck::spellcheck_subs;
pub use subtitle_file::SubtitleFile;
