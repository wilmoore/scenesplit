//! Semantic segmentation and frame selection module.

use crate::config::DetailLevel;
use crate::embeddings::{cosine_similarity, EmbeddedFrame};

/// A segment of semantically similar frames.
#[derive(Debug, Clone)]
pub struct SemanticSegment {
    pub index: usize,
    pub start_frame_idx: usize,
    pub end_frame_idx: usize,
    pub representative_frame: EmbeddedFrame,
    pub frame_count: usize,
}

impl SemanticSegment {
    /// Start timestamp in seconds.
    pub fn start_timestamp(&self) -> f64 {
        self.representative_frame.timestamp_seconds()
    }

    /// Duration of segment in frames.
    pub fn duration_frames(&self) -> usize {
        self.end_frame_idx - self.start_frame_idx
    }
}

/// Segment video frames by semantic similarity.
pub struct SemanticSegmenter {
    similarity_threshold: f32,
    min_segment_frames: usize,
}

impl SemanticSegmenter {
    /// Create a new segmenter with the given detail level.
    pub fn new(detail: DetailLevel) -> Self {
        Self {
            similarity_threshold: detail.similarity_threshold(),
            min_segment_frames: detail.min_segment_frames(),
        }
    }

    /// Segment frames into semantically coherent groups.
    ///
    /// The algorithm:
    /// 1. Start with the first frame as the current segment anchor
    /// 2. Compare each subsequent frame to the anchor
    /// 3. If similarity drops below threshold (semantic change detected),
    ///    finalize current segment and start a new one
    /// 4. Enforce minimum segment length to avoid over-segmentation
    /// 5. Select the middle frame of each segment as representative
    pub fn segment<F>(
        &self,
        embedded_frames: &[EmbeddedFrame],
        mut progress_callback: Option<F>,
    ) -> Vec<SemanticSegment>
    where
        F: FnMut(usize, usize),
    {
        if embedded_frames.is_empty() {
            return Vec::new();
        }

        let mut segments = Vec::new();
        let mut segment_start_idx = 0usize;
        let mut segment_frames: Vec<&EmbeddedFrame> = vec![&embedded_frames[0]];
        let mut anchor_embedding = embedded_frames[0].embedding.clone();

        for (i, current_frame) in embedded_frames.iter().enumerate().skip(1) {
            let similarity = cosine_similarity(&anchor_embedding, &current_frame.embedding);

            // Check if we've crossed the similarity threshold
            // AND we have enough frames in the current segment
            let is_semantic_change = similarity < self.similarity_threshold;
            let has_min_frames = segment_frames.len() >= self.min_segment_frames;

            if is_semantic_change && has_min_frames {
                // Finalize current segment
                let segment = self.create_segment(segments.len(), &segment_frames, segment_start_idx);
                segments.push(segment);

                // Start new segment
                segment_start_idx = i;
                segment_frames = vec![current_frame];
                anchor_embedding = current_frame.embedding.clone();
            } else {
                segment_frames.push(current_frame);
                // Update anchor using exponential moving average
                anchor_embedding =
                    self.update_anchor(&anchor_embedding, &current_frame.embedding);
            }

            if let Some(ref mut cb) = progress_callback {
                cb(i + 1, embedded_frames.len());
            }
        }

        // Don't forget the last segment
        if !segment_frames.is_empty() {
            let segment = self.create_segment(segments.len(), &segment_frames, segment_start_idx);
            segments.push(segment);
        }

        segments
    }

    fn create_segment(
        &self,
        index: usize,
        frames: &[&EmbeddedFrame],
        start_idx: usize,
    ) -> SemanticSegment {
        // Select middle frame as representative (deterministic selection)
        let representative_idx = frames.len() / 2;
        let representative = frames[representative_idx].clone();

        SemanticSegment {
            index,
            start_frame_idx: frames[0].index(),
            end_frame_idx: frames[frames.len() - 1].index(),
            representative_frame: representative,
            frame_count: frames.len(),
        }
    }

    fn update_anchor(&self, current_anchor: &[f32], new_embedding: &[f32]) -> Vec<f32> {
        // Exponential moving average for stability
        let alpha = 0.9f32;

        let updated: Vec<f32> = current_anchor
            .iter()
            .zip(new_embedding.iter())
            .map(|(a, b)| alpha * a + (1.0 - alpha) * b)
            .collect();

        // Re-normalize
        let norm: f32 = updated.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            updated.iter().map(|x| x / norm).collect()
        } else {
            updated
        }
    }
}

/// Select representative frames from segments in deterministic order.
pub fn deterministic_frame_selection(segments: &[SemanticSegment]) -> Vec<&EmbeddedFrame> {
    let mut sorted_segments: Vec<_> = segments.iter().collect();
    sorted_segments.sort_by_key(|s| s.index);
    sorted_segments
        .iter()
        .map(|s| &s.representative_frame)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video::Frame;

    fn create_mock_frame(index: usize, timestamp: f64) -> Frame {
        Frame {
            index,
            timestamp_seconds: timestamp,
            data: vec![0u8; 100 * 100 * 3],
            width: 100,
            height: 100,
        }
    }

    fn create_embedded_frame(index: usize, timestamp: f64, embedding: Vec<f32>) -> EmbeddedFrame {
        let frame = create_mock_frame(index, timestamp);
        // Normalize embedding
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized = if norm > 0.0 {
            embedding.iter().map(|x| x / norm).collect()
        } else {
            embedding
        };
        EmbeddedFrame {
            frame,
            embedding: normalized,
        }
    }

    #[test]
    fn test_empty_input() {
        let segmenter = SemanticSegmenter::new(DetailLevel::Summary);
        let segments = segmenter.segment::<fn(usize, usize)>(&[], None);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_single_frame() {
        let frames = vec![create_embedded_frame(0, 0.0, vec![1.0, 0.0, 0.0])];

        let segmenter = SemanticSegmenter::new(DetailLevel::Summary);
        let segments = segmenter.segment::<fn(usize, usize)>(&frames, None);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].frame_count, 1);
    }

    #[test]
    fn test_segments_ordered() {
        // Create frames with varying embeddings
        let frames: Vec<_> = (0..100)
            .map(|i| {
                let angle = (i as f32) * 0.1;
                create_embedded_frame(i, i as f64 / 30.0, vec![angle.cos(), angle.sin(), 0.0])
            })
            .collect();

        let segmenter = SemanticSegmenter::new(DetailLevel::All);
        let segments = segmenter.segment::<fn(usize, usize)>(&frames, None);

        for i in 1..segments.len() {
            assert!(segments[i].index > segments[i - 1].index);
        }
    }
}
