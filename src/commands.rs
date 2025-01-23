use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Convert text file to Utf8 char format (including BOM).
    ConvertToUtf8,
}
