//! Configuration types for SceneSplit.

use clap::ValueEnum;

/// Default output directory name.
pub const DEFAULT_OUTPUT_DIR: &str = "scenesplit_output";

/// Output image format.
pub const OUTPUT_IMAGE_FORMAT: &str = "jpg";

/// Output image quality (1-100).
pub const OUTPUT_IMAGE_QUALITY: i32 = 95;

/// Detail level controlling extraction granularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DetailLevel {
    /// Minimal: only major scene changes (5-10 frames for long video)
    Key,
    /// Moderate: representative frames (10-20 frames)
    Summary,
    /// Comprehensive: all detected semantic changes (20-30 frames)
    All,
}

impl DetailLevel {
    /// Get the cosine similarity threshold for this detail level.
    ///
    /// Lower threshold = more frames (smaller changes detected).
    /// Higher threshold = fewer frames (only major changes detected).
    pub fn similarity_threshold(self) -> f32 {
        match self {
            DetailLevel::Key => 0.92,
            DetailLevel::Summary => 0.85,
            DetailLevel::All => 0.75,
        }
    }

    /// Minimum frames between keyframes to avoid over-extraction.
    pub fn min_segment_frames(self) -> usize {
        match self {
            DetailLevel::Key => 90,     // ~3 seconds at 30fps
            DetailLevel::Summary => 45, // ~1.5 seconds
            DetailLevel::All => 15,     // ~0.5 seconds
        }
    }
}

impl Default for DetailLevel {
    fn default() -> Self {
        DetailLevel::Summary
    }
}

/// Quality preset affecting processing fidelity and speed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum QualityPreset {
    /// Quick processing, lower fidelity
    Fast,
    /// Default: good balance
    Balanced,
    /// Highest quality, slower
    Best,
}

impl QualityPreset {
    /// Sample every Nth frame for embedding computation.
    pub fn frame_sample_rate(self) -> usize {
        match self {
            QualityPreset::Fast => 15,
            QualityPreset::Balanced => 5,
            QualityPreset::Best => 1,
        }
    }

    /// Batch size for embedding computation.
    pub fn embedding_batch_size(self) -> usize {
        match self {
            QualityPreset::Fast => 64,
            QualityPreset::Balanced => 32,
            QualityPreset::Best => 16,
        }
    }

    /// Factor to resize images for embedding (1.0 = full size).
    pub fn image_resize_factor(self) -> f32 {
        match self {
            QualityPreset::Fast => 0.5,
            QualityPreset::Balanced => 0.75,
            QualityPreset::Best => 1.0,
        }
    }
}

impl Default for QualityPreset {
    fn default() -> Self {
        QualityPreset::Balanced
    }
}
