use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    num::NonZero,
};

use matroska_demuxer::{Frame, MatroskaFile, TrackType};
use subtile::time::{TimePoint, TimeSpan};

use crate::{
    file_processor::FileProcessor,
    matroska::{CodecId, SrtWriter, SubtitleLineDecoder, VobSubDecoder, WebvttWriter},
};

/// Extract subtitles from indicated files.
pub fn extract_subs(files: &FileProcessor) {
    files
        .subtitle_files()
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext == "mkv" || ext == "webm")
        })
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

    let info = mkv.info();
    println!("Media `{path:?}` :\n{info:#?}");
    let timestamp_scale = info.timestamp_scale();
    assert!(timestamp_scale == NonZero::new(1_000_000).unwrap());

    let filestem = path.file_stem().unwrap().to_string_lossy();

    let tracks = mkv.tracks();
    let (subtile_track_idx, mut tracks_info) = tracks
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
        .filter_map(|track| {
            if let Ok(codec) = CodecId::try_from(track.codec_id()) {
                Some((codec, track))
            } else {
                None
            }
        })
        .map(|(codec, track)| {
            let track_num = track.track_number().get();
            let default_duration = track.default_duration();
            let lang = track
                .language()
                .map(|lang| format!(".{lang}"))
                .unwrap_or_else(|| "".into());

            let decoder: Box<dyn SubtitleLineDecoder> = if codec == CodecId::SubRip {
                let filename = format!("{filestem:?}.{track_num}{lang}.srt"); //TODO
                let mut file = BufWriter::new(File::create(filename).unwrap());
                //TODO: write BOM in SrtWriter ?
                file.write_all(&crate::file_encoding::UTF8_BOM).unwrap();
                Box::new(SrtWriter::new(file))
            } else if codec == CodecId::WebVTT {
                //TODO: manage track data
                let filename = format!("{filestem:?}.{track_num}{lang}.vtt"); //TODO
                let file = BufWriter::new(File::create(filename).unwrap());
                let codec_private = track.codec_private();
                Box::new(WebvttWriter::new(file, codec_private))
            } else if codec == CodecId::VobSub {
                Box::new(VobSubDecoder::new(track.codec_private().unwrap()))
            } else {
                todo!()
            };
            (track_num, (decoder, default_duration))
        })
        .unzip::<_, _, Vec<_>, Vec<_>>();

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
            let default_duration = default_duration.map(|val| val.get());
            let duration = frame
                .duration
                .or(default_duration)
                .expect("no duration or default duration");
            assert!(frame.timestamp <= i64::MAX as u64);
            let time_start = TimePoint::from_msecs(frame.timestamp as i64);
            let time_end = TimePoint::from_msecs((frame.timestamp + duration) as i64);
            let time_span = TimeSpan::new(time_start, time_end);

            decoder.push_sub_line(time_span, &frame.data);
        }
    }
}
