//! Video loading and frame extraction module.

use std::path::{Path, PathBuf};

use opencv::core::Mat;
use opencv::imgproc;
use opencv::prelude::*;
use opencv::videoio::{self, VideoCapture, VideoCaptureTraitConst};

use crate::config::QualityPreset;
use crate::error::{Error, Result, SUPPORTED_FORMATS};

/// Metadata extracted from a video file.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VideoMetadata {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub frame_count: u32,
    pub duration_seconds: f64,
    pub codec: String,
}

/// A single video frame with metadata.
#[derive(Debug, Clone)]
pub struct Frame {
    pub index: usize,
    pub timestamp_seconds: f64,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Frame {
    /// Timestamp in milliseconds.
    #[allow(dead_code)]
    pub fn timestamp_ms(&self) -> u64 {
        (self.timestamp_seconds * 1000.0) as u64
    }
}

/// Video loader for extracting frames from video files.
pub struct VideoLoader {
    path: PathBuf,
    metadata: Option<VideoMetadata>,
}

impl VideoLoader {
    /// Create a new video loader.
    ///
    /// # Errors
    ///
    /// Returns an error if the file doesn't exist or has an unsupported format.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        Self::validate_file(&path)?;

        Ok(Self {
            path,
            metadata: None,
        })
    }

    fn validate_file(path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(Error::VideoNotFound(path.to_path_buf()));
        }

        if !path.is_file() {
            return Err(Error::VideoNotFound(path.to_path_buf()));
        }

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !SUPPORTED_FORMATS.contains(&extension.as_str()) {
            return Err(Error::UnsupportedFormat {
                path: path.to_path_buf(),
                extension,
            });
        }

        Ok(())
    }

    fn open_capture(&self) -> Result<VideoCapture> {
        let cap = VideoCapture::from_file(self.path.to_str().unwrap(), videoio::CAP_ANY)?;

        if !cap.is_opened()? {
            return Err(Error::VideoDecode {
                path: self.path.clone(),
                reason: "Failed to open video file".to_string(),
            });
        }

        Ok(cap)
    }

    /// Get video metadata, loading it if necessary.
    pub fn metadata(&mut self) -> Result<&VideoMetadata> {
        if self.metadata.is_none() {
            let cap = self.open_capture()?;

            let width = cap.get(videoio::CAP_PROP_FRAME_WIDTH)? as u32;
            let height = cap.get(videoio::CAP_PROP_FRAME_HEIGHT)? as u32;
            let fps = cap.get(videoio::CAP_PROP_FPS)?;
            let frame_count = cap.get(videoio::CAP_PROP_FRAME_COUNT)? as u32;
            let fourcc = cap.get(videoio::CAP_PROP_FOURCC)? as u32;

            // Decode fourcc to string
            let codec = (0..4)
                .map(|i| ((fourcc >> (8 * i)) & 0xFF) as u8 as char)
                .collect::<String>()
                .trim()
                .to_string();

            let duration = if fps > 0.0 {
                frame_count as f64 / fps
            } else {
                0.0
            };

            self.metadata = Some(VideoMetadata {
                path: self.path.clone(),
                width,
                height,
                fps,
                frame_count,
                duration_seconds: duration,
                codec,
            });
        }

        Ok(self.metadata.as_ref().unwrap())
    }

    /// Extract frames from the video at the specified sample rate.
    pub fn extract_frames<F>(
        &mut self,
        quality: QualityPreset,
        mut progress_callback: Option<F>,
    ) -> Result<Vec<Frame>>
    where
        F: FnMut(usize, usize),
    {
        let metadata = self.metadata()?.clone();
        let mut cap = self.open_capture()?;

        let sample_rate = quality.frame_sample_rate();
        let total_frames = metadata.frame_count as usize;
        let fps = metadata.fps;

        let mut frames = Vec::new();
        let mut frame_mat = Mat::default();
        let mut frame_index = 0usize;

        loop {
            let ret = cap.read(&mut frame_mat)?;
            if !ret || frame_mat.empty() {
                break;
            }

            // Only process frames at the sample rate
            if frame_index.is_multiple_of(sample_rate) {
                let timestamp = if fps > 0.0 {
                    frame_index as f64 / fps
                } else {
                    0.0
                };

                // Convert BGR to RGB
                let mut rgb_mat = Mat::default();
                imgproc::cvt_color_def(&frame_mat, &mut rgb_mat, imgproc::COLOR_BGR2RGB)?;

                // Get frame dimensions
                let width = rgb_mat.cols() as u32;
                let height = rgb_mat.rows() as u32;

                // Convert to Vec<u8>
                let data = mat_to_vec(&rgb_mat)?;

                frames.push(Frame {
                    index: frame_index,
                    timestamp_seconds: timestamp,
                    data,
                    width,
                    height,
                });
            }

            frame_index += 1;

            if let Some(ref mut cb) = progress_callback {
                cb(frame_index, total_frames);
            }
        }

        Ok(frames)
    }

    /// Get a specific frame by index.
    #[allow(dead_code)]
    pub fn get_frame_at(&mut self, index: usize) -> Result<Frame> {
        let metadata = self.metadata()?.clone();

        if index >= metadata.frame_count as usize {
            return Err(Error::VideoDecode {
                path: self.path.clone(),
                reason: format!(
                    "Frame index {} out of range [0, {})",
                    index, metadata.frame_count
                ),
            });
        }

        let mut cap = self.open_capture()?;
        cap.set(videoio::CAP_PROP_POS_FRAMES, index as f64)?;

        let mut frame_mat = Mat::default();
        let ret = cap.read(&mut frame_mat)?;

        if !ret || frame_mat.empty() {
            return Err(Error::VideoDecode {
                path: self.path.clone(),
                reason: format!("Failed to read frame at index {}", index),
            });
        }

        let timestamp = if metadata.fps > 0.0 {
            index as f64 / metadata.fps
        } else {
            0.0
        };

        // Convert BGR to RGB
        let mut rgb_mat = Mat::default();
        imgproc::cvt_color_def(&frame_mat, &mut rgb_mat, imgproc::COLOR_BGR2RGB)?;

        let width = rgb_mat.cols() as u32;
        let height = rgb_mat.rows() as u32;
        let data = mat_to_vec(&rgb_mat)?;

        Ok(Frame {
            index,
            timestamp_seconds: timestamp,
            data,
            width,
            height,
        })
    }

    /// Get the path to the video file.
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Convert an OpenCV Mat to a Vec<u8>.
fn mat_to_vec(mat: &Mat) -> Result<Vec<u8>> {
    let total = mat.total() * mat.channels() as usize;
    let mut data = vec![0u8; total];

    // Copy data from Mat
    let mat_data = mat.data_bytes()?;
    data.copy_from_slice(mat_data);

    Ok(data)
}
