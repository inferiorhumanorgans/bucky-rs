use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuckyError {
    #[error("domain must not include 0")]
    DegenerateDomain,

    #[error("descending scale not allowed")]
    DescendingScale,

    #[error("Date/Time parsing error: {0}")]
    ChronoParseError(#[from] chrono::ParseError),
}

pub type Result<T> = std::result::Result<T, BuckyError>;
