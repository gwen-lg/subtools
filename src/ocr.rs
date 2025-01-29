use std::{
    fs::File,
    io::{BufReader, Cursor, Write},
    str::Utf8Error,
};

use image::{DynamicImage, GrayImage};
use leptess::{
    leptonica,
    tesseract::{TessInitError, TessSetVariableError},
    LepTess, Variable,
};
use subtile::{
    image::{luma_a_to_luma, ToOcrImage, ToOcrImageOpt},
    pgs,
    time::TimeSpan,
    vobsub,
};
use thiserror::Error;

use crate::{
    file_processor::FileProcessor,
    subtitle_file::{SubtitleFile, SubtitleFormat},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to create Ocr instance with '{data_path}' and '{lang}'")]
    CreateLepTess {
        #[source]
        source: TessInitError,
        data_path: String,
        lang: String,
    },

    #[error("Failed to set variable on `LepTess` instance.")]
    SetVariable(#[source] TessSetVariableError),

    #[error("Failed to set image for ORC.")]
    SetImage(#[source] leptonica::PixError),

    #[error("Failed to get text from image.")]
    GetUtf8Text(#[source] Utf8Error),

    #[error("Can't parse Subtitle format '{format}', feature is not implemented")]
    CantParseSubtitleFormat { format: SubtitleFormat },

    #[error("Failed to open Index file.")]
    IndexOpen(#[source] vobsub::VobSubError),

    #[error("Failed to create PgsParser from file")]
    PgsParserFromFile(#[source] pgs::PgsError),

    #[error("Failed to parse Pgs")]
    PgsParsing(#[source] pgs::PgsError),
}

pub fn ocr_subs(files: &FileProcessor) {
    let mut tess_wrapper = TessWrapper::new(None, "eng").unwrap(); //TODO: data_path and lang

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
                ocr_sub(&mut tess_wrapper, sub_file).unwrap();
            }
        });
}

fn ocr_sub(tess: &mut TessWrapper, file: SubtitleFile) -> Result<(), Error> {
    let (times, images) = match file.format() {
        SubtitleFormat::Pgs => {
            let parser = {
                pgs::SupParser::<BufReader<File>, pgs::DecodeTimeImage>::from_file(&file.path())
                    .map_err(Error::PgsParserFromFile)?
            };

            let (times, rle_images) = {
                parser
                    .collect::<Result<(Vec<_>, Vec<_>), _>>()
                    .map_err(Error::PgsParsing)?
            };
            let conv_fn = luma_a_to_luma::<_, _, 100, 100>; // Hardcoded value for alpha and luma threshold than work not bad.

            let images = {
                let ocr_opt = ToOcrImageOpt::default(); //TODO: allow customisation
                rle_images
                    .iter()
                    .map(|rle_img| pgs::RleToImage::new(rle_img, &conv_fn).image(&ocr_opt))
                    .collect::<Vec<_>>()
            };

            (times, images)
        }
        SubtitleFormat::VobSub => {
            let idx = { vobsub::Index::open(&file.path()).map_err(Error::IndexOpen)? };
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
                let palette = rgb_palette_to_luminance(idx.palette());
                images
                    .iter()
                    .map(|vobsub_img| {
                        let converter = vobsub::VobSubOcrImage::new(vobsub_img, &palette);
                        converter.image(&ocr_opt)
                    })
                    .collect::<Vec<_>>()
            };

            (times, images_for_ocr)
        }
        _ => {
            return Err(Error::CantParseSubtitleFormat {
                format: file.format(),
            })
        }
    };

    let texts = images
        .into_iter()
        .map(|image| {
            let text = tess.text_from_image(image, 150);
            text //TODO: use configurable dpi
        })
        .collect::<Vec<_>>();

    //TODO: write file in path
    let mut file = File::create(path).unwrap();
    file.write_all("TODO".as_bytes()).unwrap();

    Ok(())
}

/// Convert an sRGB palette to a luminance palette.
#[must_use]
pub fn rgb_palette_to_luminance(palette: &vobsub::Palette) -> [f32; 16] {
    palette.map(|x| {
        let r = srgb_to_linear(x[0]);
        let g = srgb_to_linear(x[1]);
        let b = srgb_to_linear(x[2]);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    })
}

/// Convert an sRGB color space channel to linear.
fn srgb_to_linear(channel: u8) -> f32 {
    let value = f32::from(channel) / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

struct TessWrapper(LepTess);

impl TessWrapper {
    fn new(data_path: Option<&str>, lang: impl AsRef<str>) -> Result<Self, Error> {
        let lep_tess =
            LepTess::new(data_path, lang.as_ref()).map_err(|source| Error::CreateLepTess {
                source,
                data_path: data_path.unwrap_or("<None>").into(),
                lang: lang.as_ref().into(),
            })?;
        //Set default config

        Ok(Self(lep_tess))
    }

    fn set_config(&mut self, config: &[(Variable, String)]) -> Result<(), Error> {
        for (key, value) in config {
            self.0
                .set_variable(*key, value)
                .map_err(Error::SetVariable)?;
        }
        Ok(())
    }

    fn text_from_image(&mut self, image: GrayImage, dpi: i32) -> Result<String, Error> {
        let bytes = {
            //profiling::scope!("TesseractWrapper Pnm create");
            let mut bytes: Cursor<Vec<u8>> = Cursor::new(Vec::new());
            DynamicImage::ImageLuma8(image).write_to(&mut bytes, image::ImageFormat::Pnm)?;
            bytes
        };
        self.0
            .set_image_from_mem(bytes.get_ref())
            .map_err(Error::SetImage)?;
        self.0.set_source_resolution(dpi);

        let text = self.0.get_utf8_text().map_err(Error::GetUtf8Text)?;
        Ok(text)
    }
}
