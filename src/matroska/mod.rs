//! specific code for matroska subtitle management

mod codec_id;

pub use codec_id::CodecId;
use subtile::time::TimeSpan;

/// Define the interface for manage (decode) a subtitle line from source (like matroska)
pub trait SubtitleLineDecoder {
    /// Call with information/data on a subtitle line.
    fn push_sub_line(&mut self, time: TimeSpan, content: &[u8]);
}
