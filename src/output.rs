//! Output generation module for extracted frames and metadata.

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use image::{ImageBuffer, Rgb};
use serde::Serialize;

use crate::config::{DEFAULT_OUTPUT_DIR, OUTPUT_IMAGE_FORMAT, OUTPUT_IMAGE_QUALITY};
use crate::error::{Error, Result};
use crate::segmentation::SemanticSegment;
use crate::video::VideoMetadata;

/// Metadata for a single extracted frame.
#[derive(Debug, Clone, Serialize)]
pub struct FrameMetadata {
    pub filename: String,
    pub segment_index: usize,
    pub frame_index: usize,
    pub timestamp_seconds: f64,
    pub timestamp_formatted: String,
}

/// Complete metadata for an extraction run.
#[derive(Debug, Clone, Serialize)]
pub struct OutputMetadata {
    pub source_video: String,
    pub video_duration_seconds: f64,
    pub video_frame_count: u32,
    pub extracted_frames: usize,
    pub detail_level: String,
    pub quality_preset: String,
    pub frames: Vec<FrameMetadata>,
}

/// Write extracted frames and metadata to disk.
pub struct OutputWriter {
    output_dir: PathBuf,
}

impl OutputWriter {
    /// Create a new output writer.
    pub fn new(output_dir: Option<PathBuf>) -> Self {
        let output_dir = output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT_DIR));
        Self { output_dir }
    }

    /// Create the output directory if it doesn't exist.
    pub fn prepare(&self) -> Result<&Path> {
        fs::create_dir_all(&self.output_dir).map_err(|e| {
            Error::Output(format!(
                "Failed to create output directory '{}': {}",
                self.output_dir.display(),
                e
            ))
        })?;
        Ok(&self.output_dir)
    }

    /// Write a frame image to disk.
    pub fn write_frame(
        &self,
        segment: &SemanticSegment,
        frame_number: usize,
    ) -> Result<FrameMetadata> {
        let filename = format!("{:04}.{}", frame_number, OUTPUT_IMAGE_FORMAT);
        let filepath = self.output_dir.join(&filename);

        let frame = &segment.representative_frame.frame;

        // Create image from RGB data
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_raw(frame.width, frame.height, frame.data.clone()).ok_or_else(
                || {
                    Error::Output(format!(
                        "Failed to create image buffer for frame {}",
                        frame_number
                    ))
                },
            )?;

        // Save as JPEG with quality setting
        let file = File::create(&filepath)?;
        let writer = BufWriter::new(file);

        let mut encoder =
            image::codecs::jpeg::JpegEncoder::new_with_quality(writer, OUTPUT_IMAGE_QUALITY as u8);
        encoder
            .encode_image(&img)
            .map_err(|e| Error::Output(format!("Failed to encode frame: {}", e)))?;

        Ok(FrameMetadata {
            filename,
            segment_index: segment.index,
            frame_index: frame.index,
            timestamp_seconds: frame.timestamp_seconds,
            timestamp_formatted: format_timestamp(frame.timestamp_seconds),
        })
    }

    /// Write all segment representative frames to disk.
    pub fn write_frames<F>(
        &self,
        segments: &[SemanticSegment],
        mut progress_callback: Option<F>,
    ) -> Result<Vec<FrameMetadata>>
    where
        F: FnMut(usize, usize),
    {
        self.prepare()?;
        let mut frame_metadata = Vec::with_capacity(segments.len());

        for (i, segment) in segments.iter().enumerate() {
            let metadata = self.write_frame(segment, i + 1)?;
            frame_metadata.push(metadata);

            if let Some(ref mut cb) = progress_callback {
                cb(i + 1, segments.len());
            }
        }

        Ok(frame_metadata)
    }

    /// Write extraction metadata to a JSON file.
    pub fn write_metadata(
        &self,
        video_metadata: &VideoMetadata,
        frame_metadata: Vec<FrameMetadata>,
        detail_level: &str,
        quality_preset: &str,
    ) -> Result<PathBuf> {
        let output_meta = OutputMetadata {
            source_video: video_metadata.path.to_string_lossy().to_string(),
            video_duration_seconds: video_metadata.duration_seconds,
            video_frame_count: video_metadata.frame_count,
            extracted_frames: frame_metadata.len(),
            detail_level: detail_level.to_string(),
            quality_preset: quality_preset.to_string(),
            frames: frame_metadata,
        };

        let metadata_path = self.output_dir.join("metadata.json");
        let file = File::create(&metadata_path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &output_meta)
            .map_err(|e| Error::Output(format!("Failed to write metadata: {}", e)))?;

        Ok(metadata_path)
    }

    /// Get the output directory path.
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

/// Format a timestamp as HH:MM:SS.mmm.
fn format_timestamp(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = seconds % 60.0;
    format!("{:02}:{:02}:{:06.3}", hours, minutes, secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0.0), "00:00:00.000");
        assert_eq!(format_timestamp(61.5), "00:01:01.500");
        assert_eq!(format_timestamp(3661.123), "01:01:01.123");
    }
}
