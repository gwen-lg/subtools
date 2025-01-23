//! `subtools` is a library line app to check and manipulate subtitles.
//! A command line utilities is also provided.

mod file_processor;
mod subtitle_file;

pub use file_processor::FileProcessor;
pub use subtitle_file::SubtitleFile;
