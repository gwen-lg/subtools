use std::{
    ffi::{OsStr, OsString},
    fmt, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// The error type for [`SubtitleFile`]
#[derive(Debug, Error)]
pub enum SubtitleFileError {
    #[error("the extension `{file_ext:?}` is not utf-8 compatible.")]
    NotUft8Extension { file_ext: OsString },

    #[error("Failed to canonicalize '{path:?}'")]
    Canonicalize {
        #[source]
        source: io::Error,
        path: PathBuf,
    },

    #[error("the extension `{0:?}` is not recognized as subtitle managed format.")]
    NotRecognized(OsString),

    #[error("no extension to get subtitle format")]
    MissingExtension,
}

/// Enumeration of different recognized subtitles file formats.
#[derive(Clone, Copy, Debug)]
pub enum SubtitleFormat {
    /// HDMV PGS,
    /// File extension : `.sup`
    Pgs,
    /// Subrip
    /// File extension : `.srt`
    Srt,
    /// Files extension : `.idx` + `.sub`
    /// `idx` file is optional.
    VobSub,
    /// `Web Video Text Tracks` subtitle format used by `<track>` of HTML.
    /// File extension : `.vtt`
    WebVtt,
}

impl SubtitleFormat {
    /// Indicate if the format is base on text (unlike binary format)
    pub const fn is_text(self) -> bool {
        match self {
            Self::Srt | Self::WebVtt => true,
            Self::Pgs | Self::VobSub => false,
        }
    }
    pub const fn is_image(self) -> bool {
        match self {
            Self::Srt | Self::WebVtt => false,
            Self::Pgs | Self::VobSub => true,
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "srt" => Some(Self::Srt),
            "sub" => Some(Self::VobSub),
            "sup" => Some(Self::Pgs),
            "vtt" => Some(Self::WebVtt),
            _ => None,
        }
    }
}

impl fmt::Display for SubtitleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Struct to handle information on a Subtitle file.
pub struct SubtitleFile {
    path: PathBuf,
    //lang: Option<String>,
    format: SubtitleFormat,
}

impl SubtitleFile {
    /// Get full path of the file
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
    /// return the filename part of the file path.
    #[must_use]
    pub fn filename(&self) -> Option<&OsStr> {
        self.path.file_name()
    }
    /// Indicate if the file correspond to a Subtitle text format.
    #[must_use]
    pub const fn is_text(&self) -> bool {
        self.format.is_text()
    }
    /// Indicate if the file correspond to a Subtitle image format.
    #[must_use]
    pub const fn is_image(&self) -> bool {
        self.format.is_image()
    }
    /// Return the format of the subtitle file.
    #[must_use]
    pub const fn format(&self) -> SubtitleFormat {
        self.format
    }

    /// Generate a name for a new file associated with this.
    #[expect(clippy::missing_panics_doc)]
    #[must_use]
    pub fn gen_new_name(&self, pre_ext: &str) -> PathBuf {
        //TODO: manage lang separate with `.`
        let file_stem = self.path.file_stem().unwrap().to_str().unwrap().to_owned();
        let ext = self.path.extension().unwrap().to_str().unwrap();
        let new_filename = format!("{file_stem}.{pre_ext}.{ext}");
        self.path.with_file_name(new_filename)
    }
}

impl<'a> TryFrom<&'a Path> for SubtitleFile {
    type Error = SubtitleFileError;

    fn try_from(path: &'a Path) -> Result<Self, Self::Error> {
        if let Some(file_ext) = path.extension() {
            let ext = file_ext
                .to_str()
                .ok_or_else(|| SubtitleFileError::NotUft8Extension {
                    file_ext: file_ext.to_os_string(),
                })?;
            if let Some(format) = SubtitleFormat::from_extension(ext) {
                let path =
                    path.canonicalize()
                        .map_err(|source| SubtitleFileError::Canonicalize {
                            source,
                            path: path.to_path_buf(),
                        })?;
                Ok(Self { path, format })
            } else {
                Err(SubtitleFileError::NotRecognized(file_ext.to_os_string()))
            }
        } else {
            //TODO: mange no extension ?
            Err(SubtitleFileError::MissingExtension)
        }
    }
}
