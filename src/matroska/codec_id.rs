use thiserror::Error;

/// List of codec recognized
#[derive(Debug, PartialEq, Eq)]
pub enum CodecId {
    /// The format of subtitle in DVD. It'a an image based subtitle format.
    VobSub,
    /// The format of subtitle in Bluray. It'a an image based subtitle format.
    Pgs,
    /// A very simple subtitle text format.
    SubRip,
    /// TODO
    WebVTT,
    /// TODO
    Ass,
}

#[derive(Debug, Error)]
pub enum CodecIdFromError {
    #[error("The codec_id `0` is not knowrecognized.")]
    CodecIdNotManaged(String), // TODO: use ArrayString
}
impl TryFrom<&str> for CodecId {
    type Error = CodecIdFromError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            CODECID_VOBSUB => Ok(Self::VobSub),
            CODECID_PGS => Ok(Self::Pgs),
            CODECID_SUBRIP => Ok(Self::SubRip),
            CODECID_WEBVTT => Ok(Self::WebVTT),
            CODECID_ASS => Ok(Self::Ass),
            val => Err(CodecIdFromError::CodecIdNotManaged(val.into())),
        }
    }
}

/// TODO: use SubtitleFormat as base of CodecId
//impl From<SubtitleFormat> for CodecId {}

const CODECID_VOBSUB: &str = "S_VOBSUB";
const CODECID_PGS: &str = "S_HDMV/PGS";
const CODECID_SUBRIP: &str = "S_TEXT/UTF8";
const CODECID_WEBVTT: &str = "D_WEBVTT/SUBTITLES";
const CODECID_ASS: &str = "S_TEXT/ASS";

impl CodecId {
    /// TODO
    #[must_use]
    pub const fn id_str(&self) -> &'static str {
        match self {
            Self::VobSub => CODECID_VOBSUB,
            Self::Pgs => CODECID_PGS,
            Self::SubRip => CODECID_SUBRIP,
            Self::WebVTT => CODECID_WEBVTT,
            Self::Ass => CODECID_ASS,
        }
    }
    // const fn ext(&self) -> &str {
    //     // const CODEC_VOBSUB:  ext: "sub" + "idx"
    //     // const CODEC_PGS:     ext: "sup",
    //     // const CODEC_SUBRIP:  ext: "srt",
    //     // const CODEC_WEBVTT:  ext: "vtt",
    //     // const CODEC_ASS:     ext: "ass",
    // }
}
