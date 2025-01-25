use std::{
    ffi::OsStr,
    fmt,
    path::{Path, PathBuf},
};

/// Enumeration of different recognized subtitles file formats.
#[derive(Clone, Copy, Debug)]
pub enum SubtitleFormat {
    /// Subrip
    /// File extension : `.srt`
    Srt,
    /// File extension : `.idx` + `.sub`
    /// `idx` file is optional.
    VobSub,
    /// HDMV PGS,
    /// File extension : `.sup`
    Pgs,
    /// `Web Video Text Tracks` subtitle format used by `<track>` of HTML.
    /// File extension : `.vtt`
    WebVtt,
}

impl SubtitleFormat {
    /// Indicate if the format is base on text (unlike binary format)
    pub const fn is_text(self) -> bool {
        match self {
            Self::Srt | Self::WebVtt => true,
            Self::VobSub | Self::Pgs => false,
        }
    }
    pub const fn is_image(self) -> bool {
        match self {
            Self::Srt | Self::WebVtt => false,
            Self::VobSub | Self::Pgs => true,
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
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
    /// return the filename part of the file path.
    pub fn filename(&self) -> Option<&OsStr> {
        self.path.file_name()
    }
    /// Indicate if the file correspond to a Subtitle text format.
    pub const fn is_text(&self) -> bool {
        self.format.is_text()
    }
    /// Indicate if the file correspond to a Subtitle image format.
    pub const fn is_image(&self) -> bool {
        self.format.is_image()
    }
    /// Return the format of the subtitle file.
    pub const fn format(&self) -> SubtitleFormat {
        self.format
    }
}

impl<'a> TryFrom<&'a Path> for SubtitleFile {
    type Error = String; //TODO

    fn try_from(path: &'a Path) -> Result<Self, Self::Error> {
        if let Some(file_ext) = path.extension() {
            let ext = file_ext
                .to_str()
                .ok_or_else(|| format!("The extension `{file_ext:?}` is not utf-8 compatible."))?;
            if let Some(format) = SubtitleFormat::from_extension(ext) {
                let path = path
                    .canonicalize()
                    .map_err(|err| format!("Failed to canonicalize '{path:?}' : {err}"))?;
                Ok(Self { path, format })
            } else {
                Err(format!(
                    "The extension `{file_ext:?}` is not recognized as subtitle managed format."
                ))
            }
        } else {
            //TODO: mange no extension ?
            Err("No extension to get subtitle format".into())
        }
    }
}
