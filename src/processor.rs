//! Main processing pipeline for SceneSplit.

use std::path::{Path, PathBuf};

use crate::config::{DetailLevel, QualityPreset};
use crate::embeddings::EmbeddingModel;
use crate::error::Result;
use crate::output::OutputWriter;
use crate::segmentation::SemanticSegmenter;
use crate::video::{VideoLoader, VideoMetadata};

/// Result of video processing.
#[derive(Debug)]
pub struct ProcessingResult {
    pub video_metadata: VideoMetadata,
    pub total_frames_processed: usize,
    pub segments_detected: usize,
    pub frames_extracted: usize,
    pub output_dir: PathBuf,
    pub metadata_path: PathBuf,
}

/// Progress callback type for processing stages.
pub type ProgressCallback = Box<dyn FnMut(&str, usize, usize)>;

/// Main processing pipeline for semantic keyframe extraction.
pub struct SceneSplitProcessor {
    detail: DetailLevel,
    quality: QualityPreset,
    output_dir: Option<PathBuf>,
    model_path: PathBuf,
}

impl SceneSplitProcessor {
    /// Create a new processor.
    ///
    /// # Arguments
    ///
    /// * `detail` - Detail level for granularity control.
    /// * `quality` - Quality preset for performance/fidelity tradeoff.
    /// * `output_dir` - Optional output directory path.
    /// * `model_path` - Path to the ONNX model file.
    pub fn new(
        detail: DetailLevel,
        quality: QualityPreset,
        output_dir: Option<PathBuf>,
        model_path: PathBuf,
    ) -> Self {
        Self {
            detail,
            quality,
            output_dir,
            model_path,
        }
    }

    /// Process a video file and extract semantic keyframes.
    pub fn process<F>(
        &self,
        video_path: &Path,
        mut progress_callback: Option<F>,
    ) -> Result<ProcessingResult>
    where
        F: FnMut(&str, usize, usize),
    {
        // Stage 1: Load video
        Self::report_progress(&mut progress_callback, "Loading video", 0, 4);
        let mut video = VideoLoader::new(video_path)?;
        let video_meta = video.metadata()?.clone();

        // Stage 2: Extract frames
        Self::report_progress(&mut progress_callback, "Extracting frames", 1, 4);
        let frames = video.extract_frames::<fn(usize, usize)>(self.quality, None)?;

        // Stage 3: Compute embeddings
        Self::report_progress(&mut progress_callback, "Computing embeddings", 1, 4);
        let mut embedding_model = EmbeddingModel::new(&self.model_path, self.quality)?;
        let embedded_frames =
            embedding_model.compute_embeddings_batch::<fn(usize, usize)>(&frames, None)?;

        // Stage 4: Segment by semantic similarity
        Self::report_progress(&mut progress_callback, "Detecting semantic changes", 2, 4);
        let segmenter = SemanticSegmenter::new(self.detail);
        let segments = segmenter.segment::<fn(usize, usize)>(&embedded_frames, None);

        // Stage 5: Write output
        Self::report_progress(&mut progress_callback, "Writing output", 3, 4);
        let writer = OutputWriter::new(self.output_dir.clone());
        let frame_metadata = writer.write_frames::<fn(usize, usize)>(&segments, None)?;

        let metadata_path = writer.write_metadata(
            &video_meta,
            frame_metadata,
            &format!("{:?}", self.detail).to_lowercase(),
            &format!("{:?}", self.quality).to_lowercase(),
        )?;

        Self::report_progress(&mut progress_callback, "Complete", 4, 4);

        Ok(ProcessingResult {
            video_metadata: video_meta,
            total_frames_processed: frames.len(),
            segments_detected: segments.len(),
            frames_extracted: segments.len(),
            output_dir: writer.output_dir().to_path_buf(),
            metadata_path,
        })
    }

    fn report_progress<F>(callback: &mut Option<F>, stage: &str, current: usize, total: usize)
    where
        F: FnMut(&str, usize, usize),
    {
        if let Some(ref mut cb) = callback {
            cb(stage, current, total);
        }
    }
}
