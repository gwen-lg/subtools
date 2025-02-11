//! specific code for matroska subtitle management

mod codec_id;
mod content;
mod pgs;
mod subrip;
mod vobsub;
mod webvtt;

pub use codec_id::CodecId;
pub(crate) use content::ContentDecoder;
pub use pgs::PgsFrameHandler;
pub use subrip::SrtWriter;
pub use vobsub::VobSubFrameHandler;
pub use webvtt::WebvttWriter;

use compact_str::CompactString;
use subtile::time::{TimePoint, TimeSpan};

/// Define the interface to handle a frame from source (like matroska)
pub trait FrameHandler {
    /// Call with information/data on a subtitle line.
    fn push_frame(&mut self, timestamp: u64, duration: Option<u64>, content: &[u8]);
}

/// Create a `TimeSpan` from the frame timestamp and a duration
#[must_use]
#[expect(clippy::cast_possible_wrap)]
pub const fn frame_time_span(timestamp: u64, duration: u64) -> TimeSpan {
    assert!(timestamp <= i64::MAX as u64); //TODO: convert to Error
    assert!(timestamp + duration <= i64::MAX as u64); //TODO: convert to Error
    let time_start = TimePoint::from_msecs(timestamp as i64);
    let time_end = TimePoint::from_msecs((timestamp + duration) as i64);
    TimeSpan::new(time_start, time_end)
}

/// Define information for dump setup.
///
/// - `prefix`: a filename prefix
/// - `lang`: a lang `tag`
#[derive(Debug, Default)]
pub struct DumpInfo {
    prefix: CompactString,
    lang: CompactString,
}

impl DumpInfo {
    /// Create a new [`DumpInfo`] and define prefix and lang.
    #[must_use]
    pub fn new(prefix: &str, lang: &str) -> Self {
        Self {
            prefix: prefix.into(),
            lang: lang.into(),
        }
    }

    /// Get the asked prefix.
    #[must_use]
    pub const fn prefix(&self) -> &CompactString {
        &self.prefix
    }
    /// Get the asked lang `str`.
    #[must_use]
    pub const fn lang(&self) -> &CompactString {
        &self.lang
    }

    /// Compute lang suffix for dump filename.
    #[must_use]
    pub fn lang_suffix(&self) -> CompactString {
        if self.lang.is_empty() {
            "".into()
        } else {
            format!(".{}", self.lang).into()
        }
    }
}
