use crate::{
    IterProcessing, ProcessingContext, SubProcess,
    file_processor::FileProcessor,
    matroska::{CodecId, FrameHandler, SrtWriter, WebvttWriter},
    subtitle_file::SubtitleFormat,
};
use matroska_demuxer::{Frame, MatroskaFile, TrackType};
use std::{
    cell::RefCell,
    fs::File,
    io::{BufReader, BufWriter, Write},
    num::NonZero,
    path::{Path, PathBuf},
    rc::Rc,
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

    let info = mkv.info();
    writeln!(cur_ctx.borrow_mut(), "Media info :\n{info:#?}").unwrap();

    let timestamp_scale = info.timestamp_scale();
    assert!(timestamp_scale == NonZero::new(1_000_000).unwrap());

    let filestem = path.file_stem().unwrap().to_string_lossy();

    let tracks = mkv.tracks();
    let (subtile_track_idx, mut tracks_info) = tracks
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
        .filter_map(|(ctx, track)| {
            CodecId::try_from(track.codec_id()).map_or(None, |codec| {
                if SubtitleFormat::from(codec).is_text() {
                    Some((ctx, codec, track))
                } else {
                    None
                }
            })
        })
        .inspect(|(ctx, _, track_entry)| {
            writeln!(
                ctx.borrow_mut(),
                "Extract track [{}]: `{}` - {}",
                track_entry.track_number(),
                track_entry.codec_id(),
                track_entry.language().unwrap_or("eng"),
            )
            .unwrap();
        })
        .map(|(_, codec, track)| {
            let track_num = track.track_number().get();
            let default_duration = track.default_duration();
            let lang = track.language().unwrap_or("eng");

            let filename = PathBuf::from(format!("{filestem}.{track_num}-{lang}.tmp"));
            let decoder = create_frame_decoder(track, codec, filename);
            (track_num, (decoder, default_duration))
        })
        .unzip::<_, _, Vec<_>, Vec<_>>();

    if subtile_track_idx.is_empty() {
        writeln!(cur_ctx.borrow_mut(), "No track to extract").unwrap();
    } else {
        let mut frame = Frame::default();
        while mkv.next_frame(&mut frame).unwrap() {
            if let Some(select_idx) =
                subtile_track_idx
                    .iter()
                    .enumerate()
                    .find_map(|(select_idx, track_idx)| {
                        if *track_idx == frame.track {
                            Some(select_idx)
                        } else {
                            None
                        }
                    })
            //.map(|(_, duration)| duration.map(|val| val.get()))
            {
                let (decoder, default_duration) = &mut tracks_info[select_idx];
                let default_duration = default_duration.map(NonZero::get);
                let duration = frame.duration.or(default_duration);
                assert!(i64::try_from(frame.timestamp).is_ok());

                decoder.push_frame(frame.timestamp, duration, &frame.data);
            }
        }
    }
}

fn create_frame_decoder(
    track: &matroska_demuxer::TrackEntry,
    codec: CodecId,
    mut filename: PathBuf,
) -> Box<dyn FrameHandler> {
    match codec {
        CodecId::SubRip => {
            filename.set_extension("srt");
            let mut file = BufWriter::new(File::create(filename).unwrap());
            //TODO: write BOM in SrtWriter ?
            file.write_all(&crate::file_encoding::UTF8_BOM).unwrap();
            Box::new(SrtWriter::new(file))
        }
        CodecId::WebVTT => {
            //TODO: manage track data
            filename.set_extension("vtt");
            let file = BufWriter::new(File::create(filename).unwrap());
            let codec_private = track.codec_private();
            Box::new(WebvttWriter::new(file, codec_private))
        }
        _ => {
            todo!()
        }
    }
}
