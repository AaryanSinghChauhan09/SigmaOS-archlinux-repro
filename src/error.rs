//! Error types for the archlinux-repro component.

/// Errors that can occur during reproducibility verification.
#[derive(Debug, thiserror::Error)]
pub enum ReproError {
    /// A build run failed.
    #[error("build failed: {0}")]
    BuildFailed(String),

    /// Build output directory missing or invalid.
    #[error("invalid build output: {0}")]
    InvalidOutput(String),

    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
