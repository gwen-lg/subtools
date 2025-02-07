use crate::{
    IterProcessing, ProcessingContext, SubProcess, file_processor::FileProcessor, matroska::CodecId,
};
use matroska_demuxer::{Frame, MatroskaFile, TrackType};
use std::{cell::RefCell, fs::File, io::Write, path::PathBuf, rc::Rc, str};

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

fn extract_subs_mkv(proc_ctx: &ProcessingContext, path: &PathBuf) {
    let filename = path.file_name().unwrap();
    let cur_ctx = Rc::new(RefCell::new(
        proc_ctx.create_sub_process(format!("Extract sub for {}", filename.display())),
    ));

    let file = File::open(path.as_path()).unwrap();
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
        .map(|(_, track)| track.track_number().get())
        .collect::<Vec<_>>();

    let mut frame = Frame::default();
    while mkv.next_frame(&mut frame).unwrap() {
        if subtile_tracks.contains(&frame.track) {
            let duration = frame.duration.unwrap_or_default();
            let frame_content = str::from_utf8(&frame.data).unwrap();
            println!(
                "{}-{}>{duration}:\n{frame_content}",
                frame.track, frame.timestamp
            );
        }
    }
}
