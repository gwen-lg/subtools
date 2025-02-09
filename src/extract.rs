use std::{fs::File, io::BufReader, str};

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
    let file = BufReader::new(file);
    let mut mkv = MatroskaFile::open(file).unwrap();

    //let info = mkv.info();
    //println!("Media `{path:?}` :\n{info:#?}");

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
        .map(|track| {
            let track_num = track.track_number().get();
            let default_duration = track.default_duration();
            (track_num, default_duration)
        })
        .collect::<Vec<_>>();

    let mut frame = Frame::default();
    while mkv.next_frame(&mut frame).unwrap() {
        if let Some((_, default_duration)) = subtile_tracks
            .iter()
            .find(|(track_idx, _)| *track_idx == frame.track)
        //.map(|(_, duration)| duration.map(|val| val.get()))
        {
            let default_duration = default_duration.map(|val| val.get());
            let duration = frame
                .duration
                .or(default_duration)
                .expect("no duration or default duration");
            let frame_content = str::from_utf8(&frame.data).unwrap();
            println!(
                "{}-{}>{duration}:\n{frame_content}",
                frame.track, frame.timestamp
            );
        }
    }
}
