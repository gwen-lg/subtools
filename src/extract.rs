use crate::{
    IterProcessing, ProcessingContext, SubProcess, file_processor::FileProcessor, matroska::CodecId,
};
use matroska_demuxer::{Frame, MatroskaFile, TrackType};
use std::{
    cell::RefCell,
    fs::File,
    io::{BufReader, Write},
    num::NonZero,
    path::Path,
    rc::Rc,
    str,
};

/// Extract subtitles from indicated files.
pub fn extract_subs(proc_ctx: ProcessingContext, files: &FileProcessor) {
    files
        .subtitle_files()
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext == "mkv" || ext == "webm")
        })
        .process_context(proc_ctx, "Extract subtitles")
        .for_each(|(ctx, path)| {
            extract_subs_mkv(&ctx.borrow(), &path);
        });
}

fn extract_subs_mkv(proc_ctx: &ProcessingContext, path: &Path) {
    let filename = path.file_name().unwrap();
    let cur_ctx = Rc::new(RefCell::new(
        proc_ctx.create_sub_process(format!("Extract sub for {}", filename.display())),
    ));

    let file = File::open(path).unwrap();
    let file = BufReader::new(file);
    let mut mkv = MatroskaFile::open(file).unwrap();

    let info = mkv.info();
    writeln!(cur_ctx.borrow_mut(), "Media `{path:?}` :\n{info:#?}").unwrap();

    let tracks = mkv.tracks();
    let subtile_tracks = tracks
        .iter()
        .filter(|track| track.track_type() == TrackType::Subtitle)
        .include_context(cur_ctx.clone())
        .inspect(|(ctx, track)| {
            writeln!(
                ctx.borrow_mut(),
                "track `{}`: {} - {:?}",
                track.track_number(),
                track.codec_id(),
                track.codec_name()
            )
            .unwrap();
        })
        .filter_map(|(_ctx, track)| {
            CodecId::try_from(track.codec_id()).map_or(None, |codec| Some((codec, track)))
        })
        .filter(|(codec, _)| codec.id_str() == "S_TEXT/UTF8")
        .map(|(_, track)| {
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
        {
            let default_duration = default_duration.map(NonZero::get);
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
