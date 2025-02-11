//! specific code for matroska subtitle management

mod codec_id;
mod subrip;

pub use codec_id::CodecId;
pub use subrip::SrtWriter;

use subtile::time::TimeSpan;

/// Define the interface for manage (decode) a subtitle line from source (like matroska)
pub trait SubtitleLineDecoder {
    /// Call with information/data on a subtitle line.
    fn push_sub_line(&mut self, time: TimeSpan, content: &[u8]);
}
