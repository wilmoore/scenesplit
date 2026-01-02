"""Semantic segmentation and frame selection module."""

from dataclasses import dataclass
from typing import Sequence

import numpy as np

from scenesplit.constants import DetailLevel
from scenesplit.embeddings import EmbeddedFrame, cosine_similarity


@dataclass(frozen=True)
class SemanticSegment:
    """A segment of semantically similar frames."""

    index: int
    start_frame_idx: int
    end_frame_idx: int
    representative_frame: EmbeddedFrame
    frame_count: int

    @property
    def start_timestamp(self) -> float:
        """Start timestamp in seconds."""
        return self.representative_frame.timestamp_seconds

    @property
    def duration_frames(self) -> int:
        """Duration of segment in frames."""
        return self.end_frame_idx - self.start_frame_idx


class SemanticSegmenter:
    """Segment video frames by semantic similarity.

    Uses embedding-based similarity to detect meaningful scene changes,
    grouping similar frames together and selecting one representative
    per segment.
    """

    def __init__(self, detail: DetailLevel = DetailLevel.SUMMARY) -> None:
        """Initialize the segmenter.

        Args:
            detail: Detail level controlling segmentation granularity.
        """
        self.detail = detail
        self._similarity_threshold = detail.similarity_threshold
        self._min_segment_frames = detail.min_segment_frames

    def segment(
        self,
        embedded_frames: Sequence[EmbeddedFrame],
        progress_callback: callable | None = None,
    ) -> list[SemanticSegment]:
        """Segment frames into semantically coherent groups.

        The algorithm:
        1. Start with the first frame as the current segment anchor
        2. Compare each subsequent frame to the anchor
        3. If similarity drops below threshold (semantic change detected),
           finalize current segment and start a new one
        4. Enforce minimum segment length to avoid over-segmentation
        5. Select the middle frame of each segment as representative

        Args:
            embedded_frames: Sequence of frames with embeddings.
            progress_callback: Optional callback(current, total) for progress.

        Returns:
            List of SemanticSegments, ordered by video timeline.
        """
        if not embedded_frames:
            return []

        segments: list[SemanticSegment] = []
        segment_start_idx = 0
        segment_frames: list[EmbeddedFrame] = [embedded_frames[0]]
        anchor_embedding = embedded_frames[0].embedding

        for i in range(1, len(embedded_frames)):
            current_frame = embedded_frames[i]
            similarity = cosine_similarity(anchor_embedding, current_frame.embedding)

            # Check if we've crossed the similarity threshold
            # AND we have enough frames in the current segment
            is_semantic_change = similarity < self._similarity_threshold
            has_min_frames = len(segment_frames) >= self._min_segment_frames

            if is_semantic_change and has_min_frames:
                # Finalize current segment
                segment = self._create_segment(
                    index=len(segments),
                    frames=segment_frames,
                    start_idx=segment_start_idx,
                )
                segments.append(segment)

                # Start new segment
                segment_start_idx = i
                segment_frames = [current_frame]
                anchor_embedding = current_frame.embedding
            else:
                segment_frames.append(current_frame)
                # Update anchor using running average for stability
                anchor_embedding = self._update_anchor(
                    anchor_embedding, current_frame.embedding, len(segment_frames)
                )

            if progress_callback is not None:
                progress_callback(i + 1, len(embedded_frames))

        # Don't forget the last segment
        if segment_frames:
            segment = self._create_segment(
                index=len(segments),
                frames=segment_frames,
                start_idx=segment_start_idx,
            )
            segments.append(segment)

        return segments

    def _create_segment(
        self,
        index: int,
        frames: list[EmbeddedFrame],
        start_idx: int,
    ) -> SemanticSegment:
        """Create a segment from a list of frames.

        Selects the middle frame as the representative.
        """
        # Select middle frame as representative (deterministic selection)
        representative_idx = len(frames) // 2
        representative = frames[representative_idx]

        return SemanticSegment(
            index=index,
            start_frame_idx=frames[0].index,
            end_frame_idx=frames[-1].index,
            representative_frame=representative,
            frame_count=len(frames),
        )

    def _update_anchor(
        self,
        current_anchor: np.ndarray,
        new_embedding: np.ndarray,
        frame_count: int,
    ) -> np.ndarray:
        """Update the anchor embedding using exponential moving average.

        This provides stability against local noise while still
        tracking gradual semantic drift.
        """
        # Use a decay factor that gives more weight to recent frames
        # but still maintains some memory of the segment start
        alpha = 0.9  # High alpha = more weight on current anchor
        updated = alpha * current_anchor + (1 - alpha) * new_embedding
        # Re-normalize
        return updated / np.linalg.norm(updated)


def deterministic_frame_selection(
    segments: Sequence[SemanticSegment],
) -> list[EmbeddedFrame]:
    """Select representative frames from segments in deterministic order.

    Args:
        segments: Sequence of semantic segments.

    Returns:
        List of representative frames, ordered by timestamp.
    """
    # Sort by segment index to ensure deterministic ordering
    sorted_segments = sorted(segments, key=lambda s: s.index)
    return [s.representative_frame for s in sorted_segments]
