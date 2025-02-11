//! `subtools` is a library line app to check and manipulate subtitles.
//! A command line utilities is also provided.

mod extract;
mod file_encoding;
mod file_processor;
pub mod matroska; //TODO: move in a crate (subtile ? or subtile-matroska ?)
mod ocr;
mod subtitle_file;

pub use extract::extract_subs;
pub use file_encoding::convert_subs_to_utf8;
pub use file_processor::FileProcessor;
pub use ocr::ocr_subs;
