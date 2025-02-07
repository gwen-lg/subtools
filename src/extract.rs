use std::{fs::File, str};

use matroska_demuxer::{Frame, MatroskaFile, TrackType};

use crate::file_processor::FileProcessor;

/// Extract subtitles from indicated files.
pub fn extract_subs(files: &FileProcessor) {
    files
        .subtitle_files()
        .filter(|path| path.extension().is_some_and(|ext| ext == "mkv"))
        //.filter_map(|path| path.as_path().extension() )
        //.filter(|sub_file| sub_file.is_image())
        .for_each(|path| {
            extract_subs_mkv(path);
        });
}

fn extract_subs_mkv(path: std::path::PathBuf) {
    let file = File::open(path.as_path()).unwrap();
    let mut mkv = MatroskaFile::open(file).unwrap();

    let info = mkv.info();
    println!("Media `{path:?}` :\n{info:#?}");

    let tracks = mkv.tracks();
    let subtile_tracks = tracks
        .iter()
        .filter(|track| track.track_type() == TrackType::Subtitle)
        .inspect(|track| {
            eprintln!(
                "track `{}`: {} - {:?}",
                track.track_number(),
                track.codec_id(),
                track.codec_name()
            );
        })
        .filter(|track| track.codec_id() == "S_TEXT/UTF8")
        .map(|track| track.track_number().get())
        .collect::<Vec<_>>();

    let mut frame = Frame::default();
    while mkv.next_frame(&mut frame).unwrap() {
        if subtile_tracks.contains(&frame.track) {
            let duration = mkv.read_duration().unwrap();
            let frame_content = str::from_utf8(&frame.data).unwrap();
            println!(
                "{}-{}>{duration}:\n{frame_content}",
                frame.track, frame.timestamp
            );
        }
    }
}
