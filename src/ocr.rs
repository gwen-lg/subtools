use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

use subtile::{
    image::{self, luma_a_to_luma, ToOcrImage, ToOcrImageOpt},
    pgs, srt,
    time::TimeSpan,
    vobsub::{self, palette_rgb_to_luminance},
};
use thiserror::Error;

use crate::{
    file_processor::FileProcessor,
    subtitle_file::{SubtitleFile, SubtitleFormat},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't parse Subtitle format '{format}', feature is not implemented")]
    CantParseSubtitleFormat { format: SubtitleFormat },

    #[error("Failed to open Index file.")]
    IndexOpen(#[source] vobsub::VobSubError),

    #[error("Failed to create PgsParser from file")]
    PgsParserFromFile(#[source] pgs::PgsError),

    #[error("Failed to parse Pgs")]
    PgsParsing(#[source] pgs::PgsError),

    #[error("Error found in subtitle text.")]
    CheckSubtitles(#[source] subtile_ocr::Error),

    #[error("TODO")]
    OcrProcess(#[source] subtile_ocr::OcrError),

    #[error("Failed to create file `{path}` to write to write Srt content.")]
    CreateOutFileSrt {
        #[source]
        source: io::Error,
        path: PathBuf,
    },

    #[error("Failed to write srt content to file.")]
    WriteSrt(#[source] io::Error),
}

/// Run ocr processing on indicates files.
#[allow(clippy::missing_panics_doc)] //TODO: replace unwrap by error management
pub fn ocr_subs(files: &FileProcessor) {
    files
        .subtitle_files()
        .filter_map(|path| SubtitleFile::try_from(path.as_path()).ok())
        .filter(|sub_file| sub_file.is_image())
        .for_each(|sub_file| {
            let mut path = sub_file.path().to_path_buf();
            path.set_extension("srt");

            if path.exists() {
                eprintln!("File '{path:?}' already exist, do not override with OCR.");
            } else {
                ocr_sub(sub_file).unwrap();
            }
        });
}

fn ocr_sub(file: SubtitleFile) -> Result<(), Error> {
    let (times, images) = match file.format() {
        SubtitleFormat::VobSub => parse_vobsub(&file)?,
        SubtitleFormat::Pgs => parse_pgs(&file)?,
        _ => {
            return Err(Error::CantParseSubtitleFormat {
                format: file.format(),
            })
        }
    };
    //TODO: &opt.tessdata_dir, opt.lang.as_str(), &opt.config, opt.dpi
    let ocr_variable = Vec::new();
    let ocr_opt = subtile_ocr::OcrOpt::new(&None, "eng", &ocr_variable, 150);
    let texts = subtile_ocr::process(images, &ocr_opt).map_err(Error::OcrProcess)?;
    let subtitles = subtile_ocr::check_subtitles(times.into_iter().zip(texts))
        .map_err(Error::CheckSubtitles)?;

    // Create subtitle file.
    let mut path_out = file.path().to_path_buf();
    path_out.set_extension("srt");
    let subtitle_file =
        File::create(path_out.as_path()).map_err(|source| Error::CreateOutFileSrt {
            source,
            path: path_out,
        })?;
    let mut stream = BufWriter::new(subtitle_file);
    // Write to file.
    srt::write_srt(&mut stream, &subtitles).map_err(Error::WriteSrt)?;

    Ok(())
}

type ParseResult = (Vec<TimeSpan>, Vec<image::GrayImage>);

fn parse_vobsub(file: &SubtitleFile) -> Result<ParseResult, Error> {
    let mut idx_path = file.path().to_path_buf();
    idx_path.set_extension("idx");
    let idx = vobsub::Index::open(idx_path).map_err(Error::IndexOpen)?;

    let (times, images): (Vec<_>, Vec<_>) = {
        idx.subtitles::<(TimeSpan, vobsub::VobSubIndexedImage)>()
            .filter_map(|sub| match sub {
                Ok(sub) => Some(sub),
                Err(e) => {
                    eprintln!(
    "warning: unable to read subtitle: {e}. (This can usually be safely ignored.)"
            );
                    None
                }
            })
            .unzip()
    };
    let images_for_ocr = {
        let ocr_opt = ToOcrImageOpt::default(); //TODO: allow customisation
        let palette = palette_rgb_to_luminance(idx.palette());
        images
            .iter()
            .map(|vobsub_img| {
                let converter = vobsub::VobSubOcrImage::new(vobsub_img, &palette);
                converter.image(&ocr_opt)
            })
            .collect::<Vec<_>>()
    };
    Ok((times, images_for_ocr))
}

fn parse_pgs(file: &SubtitleFile) -> Result<ParseResult, Error> {
    let parser = {
        pgs::SupParser::<BufReader<File>, pgs::DecodeTimeImage>::from_file(file.path())
            .map_err(Error::PgsParserFromFile)?
    };
    let (times, rle_images) = {
        parser
            .collect::<Result<(Vec<_>, Vec<_>), _>>()
            .map_err(Error::PgsParsing)?
    };
    let conv_fn = luma_a_to_luma::<_, _, 100, 100>;
    let images = {
        let ocr_opt = ToOcrImageOpt::default(); //TODO: allow customisation
        rle_images
            .iter()
            .map(|rle_img| pgs::RleToImage::new(rle_img, &conv_fn).image(&ocr_opt))
            .collect::<Vec<_>>()
    };
    Ok((times, images))
}
