"""Main processing pipeline for SceneSplit."""

from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from scenesplit.constants import DetailLevel, QualityPreset
from scenesplit.embeddings import EmbeddingModel
from scenesplit.output import OutputWriter
from scenesplit.segmentation import SemanticSegmenter, deterministic_frame_selection
from scenesplit.video import VideoLoader, VideoMetadata


@dataclass
class ProcessingResult:
    """Result of video processing."""

    video_metadata: VideoMetadata
    total_frames_processed: int
    segments_detected: int
    frames_extracted: int
    output_dir: Path
    metadata_path: Path


class SceneSplitProcessor:
    """Main processing pipeline for semantic keyframe extraction.

    This processor orchestrates the full pipeline:
    1. Video loading and frame extraction
    2. Semantic embedding computation
    3. Change detection and segmentation
    4. Representative frame selection
    5. Output generation
    """

    def __init__(
        self,
        detail: DetailLevel = DetailLevel.SUMMARY,
        quality: QualityPreset = QualityPreset.BALANCED,
        output_dir: str | Path | None = None,
    ) -> None:
        """Initialize the processor.

        Args:
            detail: Detail level for granularity control.
            quality: Quality preset for performance/fidelity tradeoff.
            output_dir: Optional output directory path.
        """
        self.detail = detail
        self.quality = quality
        self.output_dir = output_dir

    def process(
        self,
        video_path: str | Path,
        progress_callback: Callable[[str, int, int], None] | None = None,
    ) -> ProcessingResult:
        """Process a video file and extract semantic keyframes.

        Args:
            video_path: Path to the input video file.
            progress_callback: Optional callback(stage, current, total) for progress.

        Returns:
            ProcessingResult with extraction details.
        """
        # Stage 1: Load video
        self._report_progress(progress_callback, "Loading video", 0, 4)
        video = VideoLoader(video_path)
        video_meta = video.metadata

        # Stage 2: Extract and embed frames
        self._report_progress(progress_callback, "Extracting frames", 1, 4)
        frames = list(video.extract_frames(quality=self.quality))

        self._report_progress(progress_callback, "Computing embeddings", 1, 4)
        embedding_model = EmbeddingModel(quality=self.quality)
        embedded_frames = embedding_model.compute_embeddings_batch(frames)

        # Stage 3: Segment by semantic similarity
        self._report_progress(progress_callback, "Detecting semantic changes", 2, 4)
        segmenter = SemanticSegmenter(detail=self.detail)
        segments = segmenter.segment(embedded_frames)

        # Stage 4: Write output
        self._report_progress(progress_callback, "Writing output", 3, 4)
        writer = OutputWriter(output_dir=self.output_dir, source_video=video.path)
        frame_metadata = writer.write_frames(segments)
        metadata_path = writer.write_metadata(
            video_metadata=video_meta,
            frame_metadata=frame_metadata,
            detail_level=self.detail.value,
            quality_preset=self.quality.value,
        )

        self._report_progress(progress_callback, "Complete", 4, 4)

        return ProcessingResult(
            video_metadata=video_meta,
            total_frames_processed=len(frames),
            segments_detected=len(segments),
            frames_extracted=len(frame_metadata),
            output_dir=writer.output_dir,
            metadata_path=metadata_path,
        )

    def _report_progress(
        self,
        callback: Callable[[str, int, int], None] | None,
        stage: str,
        current: int,
        total: int,
    ) -> None:
        """Report progress to the callback if provided."""
        if callback is not None:
            callback(stage, current, total)
