//! specific code for matroska subtitle management

mod codec_id;
mod subrip;

pub use codec_id::CodecId;
pub use subrip::SrtWriter;

use subtile::time::{TimePoint, TimeSpan};

/// Define the interface for manage (decode) a subtitle line from source (like matroska)
pub trait SubtitleLineDecoder {
    /// Call with information/data on a subtitle line.
    fn push_sub_line(&mut self, timestamp: u64, duration: Option<u64>, content: &[u8]);
}

/// Create a `TimeSpan` from the frame timestamp and a duration
#[must_use]
pub const fn frame_time_span(timestamp: u64, duration: u64) -> TimeSpan {
    assert!(timestamp <= i64::MAX as u64); //TODO: convert to Error
    let time_start = TimePoint::from_msecs(timestamp as i64);
    let time_end = TimePoint::from_msecs((timestamp + duration) as i64);
    TimeSpan::new(time_start, time_end)
}
