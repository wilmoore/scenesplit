//! Semantic embedding computation for video frames.

use std::path::Path;

use ndarray::{s, Array4};
use ort::session::{builder::GraphOptimizationLevel, Session};

use crate::config::QualityPreset;
use crate::error::{Error, Result};
use crate::video::Frame;

/// A frame with its computed embedding vector.
#[derive(Debug, Clone)]
pub struct EmbeddedFrame {
    pub frame: Frame,
    pub embedding: Vec<f32>,
}

impl EmbeddedFrame {
    /// Original frame index.
    pub fn index(&self) -> usize {
        self.frame.index
    }

    /// Frame timestamp in seconds.
    pub fn timestamp_seconds(&self) -> f64 {
        self.frame.timestamp_seconds
    }
}

/// Compute semantic embeddings for video frames using ONNX Runtime.
pub struct EmbeddingModel {
    session: Session,
    quality: QualityPreset,
}

impl EmbeddingModel {
    /// Create a new embedding model.
    ///
    /// # Arguments
    ///
    /// * `model_path` - Path to the ONNX model file (ResNet50 or similar).
    /// * `quality` - Quality preset affecting image preprocessing.
    pub fn new<P: AsRef<Path>>(model_path: P, quality: QualityPreset) -> Result<Self> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;

        Ok(Self { session, quality })
    }

    /// Preprocess a frame for the embedding model.
    ///
    /// Resizes to 224x224, normalizes with ImageNet mean/std.
    fn preprocess_frame(&self, frame: &Frame) -> Result<Array4<f32>> {
        let resize_factor = self.quality.image_resize_factor();

        // Calculate target size after optional resize
        let (src_width, src_height) = if resize_factor < 1.0 {
            (
                (frame.width as f32 * resize_factor) as u32,
                (frame.height as f32 * resize_factor) as u32,
            )
        } else {
            (frame.width, frame.height)
        };

        // Create image from raw RGB data
        let img = image::RgbImage::from_raw(frame.width, frame.height, frame.data.clone())
            .ok_or_else(|| Error::Embedding("Failed to create image from frame data".into()))?;

        // Resize if needed
        let img = if resize_factor < 1.0 {
            image::imageops::resize(
                &img,
                src_width,
                src_height,
                image::imageops::FilterType::Triangle,
            )
        } else {
            img
        };

        // Resize to 224x224 for the model
        let img = image::imageops::resize(&img, 224, 224, image::imageops::FilterType::Triangle);

        // Convert to NCHW format with normalization
        // ImageNet mean: [0.485, 0.456, 0.406], std: [0.229, 0.224, 0.225]
        let mean = [0.485f32, 0.456, 0.406];
        let std = [0.229f32, 0.224, 0.225];

        let mut tensor = Array4::<f32>::zeros((1, 3, 224, 224));

        for y in 0..224 {
            for x in 0..224 {
                let pixel = img.get_pixel(x as u32, y as u32);
                for c in 0..3 {
                    let value = pixel[c] as f32 / 255.0;
                    let normalized = (value - mean[c]) / std[c];
                    tensor[[0, c, y, x]] = normalized;
                }
            }
        }

        Ok(tensor)
    }

    /// Compute the embedding for a single frame.
    pub fn compute_embedding(&mut self, frame: &Frame) -> Result<EmbeddedFrame> {
        let input = self.preprocess_frame(frame)?;
        let input_value = ort::value::Tensor::from_array(input)?;
        let outputs = self.session.run(ort::inputs![input_value])?;

        // Get the output tensor - new API returns (shape, data) tuple
        let (_, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| Error::Embedding(format!("Failed to extract embedding: {}", e)))?;

        // Flatten and normalize
        let flat: Vec<f32> = data.iter().cloned().collect();
        let embedding = normalize_vector(&flat);

        Ok(EmbeddedFrame {
            frame: frame.clone(),
            embedding,
        })
    }

    /// Compute embeddings for a batch of frames.
    pub fn compute_embeddings_batch<F>(
        &mut self,
        frames: &[Frame],
        mut progress_callback: Option<F>,
    ) -> Result<Vec<EmbeddedFrame>>
    where
        F: FnMut(usize, usize),
    {
        if frames.is_empty() {
            return Ok(Vec::new());
        }

        let batch_size = self.quality.embedding_batch_size();
        let mut results = Vec::with_capacity(frames.len());

        for (batch_idx, chunk) in frames.chunks(batch_size).enumerate() {
            // Process batch
            let mut batch_tensor = Array4::<f32>::zeros((chunk.len(), 3, 224, 224));

            for (i, frame) in chunk.iter().enumerate() {
                let preprocessed = self.preprocess_frame(frame)?;
                batch_tensor
                    .slice_mut(s![i, .., .., ..])
                    .assign(&preprocessed.slice(s![0, .., .., ..]));
            }

            let batch_value = ort::value::Tensor::from_array(batch_tensor)?;
            let outputs = self.session.run(ort::inputs![batch_value])?;

            let (shape, data) = outputs[0]
                .try_extract_tensor::<f32>()
                .map_err(|e| Error::Embedding(format!("Failed to extract embeddings: {}", e)))?;

            // Calculate embedding size from shape (batch_size, embedding_dim, ...)
            let embedding_size = shape.iter().skip(1).product::<i64>() as usize;

            // Extract individual embeddings from batch output
            for (i, frame) in chunk.iter().enumerate() {
                let start = i * embedding_size;
                let end = start + embedding_size;
                let flat: Vec<f32> = data
                    .iter()
                    .skip(start)
                    .take(embedding_size)
                    .cloned()
                    .collect();
                let embedding = normalize_vector(&flat);

                results.push(EmbeddedFrame {
                    frame: frame.clone(),
                    embedding,
                });
            }

            if let Some(ref mut cb) = progress_callback {
                cb(results.len(), frames.len());
            }
        }

        Ok(results)
    }
}

/// Normalize a vector to unit length.
fn normalize_vector(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        v.iter().map(|x| x / norm).collect()
    } else {
        v.to_vec()
    }
}

/// Compute cosine similarity between two normalized embedding vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&v1, &v2).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v1, &v2) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_vector() {
        let v = vec![3.0, 4.0];
        let normalized = normalize_vector(&v);
        let norm: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }
}
