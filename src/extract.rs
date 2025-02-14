use crate::{
    IterProcessing, ProcessingContext, SubProcess,
    file_processor::FileProcessor,
    matroska::{
        CodecId, ContentDecoder, FrameHandler, PgsFrameHandler, SrtWriter, VobSubFrameHandler,
        WebvttWriter, write_info,
    },
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
    write_info(&mut *cur_ctx.borrow_mut(), info);

    let timestamp_scale = info.timestamp_scale();
    assert!(timestamp_scale == NonZero::new(1_000_000).unwrap());

    let filestem = path.file_stem().unwrap().to_string_lossy();

    let tracks = mkv.tracks();
    let (subtile_track_idx, mut tracks_info) = tracks
        .iter()
        .filter(|track| track.track_type() == TrackType::Subtitle)
        .include_context(cur_ctx.clone())
        .map(|(ctx, track)| {
            let codec = CodecId::try_from(track.codec_id()).ok();
            (ctx, track, codec)
        })
        //TODO: add parametric filter, like on lang or subtitle format
        .inspect(|(ctx, track_entry, codec)| {
            writeln!(
                ctx.borrow_mut(),
                "track [{}]: `{}` - {} : {}",
                track_entry.track_number(),
                track_entry.codec_id(),
                track_entry.language().unwrap_or("eng"),
                if codec.is_some() { "extract" } else { "skip" }
            )
            .unwrap();
        })
        .filter_map(|(ctx, track, codec)| codec.map(|codec| (ctx, track, codec)))
        .map(|(_, track, codec)| {
            let track_num = track.track_number().get();
            let default_duration = track.default_duration();
            let lang = track.language().unwrap_or("eng");
            let encoding = track.content_encodings();
            let content_decoder = ContentDecoder::new(encoding);

            let filename = PathBuf::from(format!("{filestem}.{track_num}-{lang}.tmp"));
            let decoder = create_frame_decoder(track, codec, filename);
            (track_num, (content_decoder, decoder, default_duration))
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
            {
                let (frame_decoder, decoder, default_duration) = &mut tracks_info[select_idx];
                let default_duration = default_duration.map(NonZero::get);
                let duration = frame.duration.or(default_duration);
                assert!(i64::try_from(frame.timestamp).is_ok());
                let content = frame_decoder.transform(frame.data.clone()); //TODO: remove the clone
                decoder.push_frame(frame.timestamp, duration, &content);
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
        CodecId::Ass => todo!(),
        CodecId::Pgs => {
            filename.set_extension("sup");
            let file = BufWriter::new(File::create(filename).unwrap());
            Box::new(PgsFrameHandler::new(file))
        }
        CodecId::SubRip => {
            filename.set_extension("srt");
            let mut file = BufWriter::new(File::create(filename).unwrap());
            //TODO: write BOM in SrtWriter ?
            file.write_all(&crate::file_encoding::UTF8_BOM).unwrap();
            Box::new(SrtWriter::new(file))
        }
        CodecId::VobSub => {
            filename.set_extension("idx");
            let file_idx = BufWriter::new(File::create(&filename).unwrap());
            filename.set_extension("sub");
            let file_sub = BufWriter::new(File::create(&filename).unwrap());
            Box::new(VobSubFrameHandler::new(
                file_idx,
                file_sub,
                track.codec_private().unwrap(),
            ))
        }
        CodecId::WebVTT => {
            //TODO: manage track data
            filename.set_extension("vtt");
            let file = BufWriter::new(File::create(filename).unwrap());
            let codec_private = track.codec_private();
            Box::new(WebvttWriter::new(file, codec_private))
        }
    }
}
