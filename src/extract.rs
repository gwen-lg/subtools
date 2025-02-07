use crate::file_processor::FileProcessor;

/// Extract subtitles from indicated files.
pub fn extract_subs(files: &FileProcessor) {
    files
        .subtitle_files()
        .filter(|path| path.extension().is_some_and(|ext| ext == "mkv"))
        .for_each(|path| {
            println!("Media file : {path:?}");
        });
}
