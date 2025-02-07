use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Convert text file to Utf8 char format (including BOM).
    ConvertToUtf8,

    /// Convert image subtitle to text subtitle with Ocr.
    Ocr,

    /// Extract subtitle track
    Extract,
}
