"""Output generation module for extracted frames and metadata."""

import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Sequence

import cv2

from scenesplit.constants import (
    DEFAULT_OUTPUT_DIR,
    OUTPUT_IMAGE_FORMAT,
    OUTPUT_IMAGE_QUALITY,
)
from scenesplit.exceptions import OutputError
from scenesplit.segmentation import SemanticSegment
from scenesplit.video import VideoMetadata


@dataclass
class FrameMetadata:
    """Metadata for a single extracted frame."""

    filename: str
    segment_index: int
    frame_index: int
    timestamp_seconds: float
    timestamp_formatted: str


@dataclass
class OutputMetadata:
    """Complete metadata for an extraction run."""

    source_video: str
    video_duration_seconds: float
    video_frame_count: int
    extracted_frames: int
    detail_level: str
    quality_preset: str
    frames: list[FrameMetadata]


class OutputWriter:
    """Write extracted frames and metadata to disk."""

    def __init__(
        self,
        output_dir: str | Path | None = None,
        source_video: Path | None = None,
    ) -> None:
        """Initialize the output writer.

        Args:
            output_dir: Directory for output files. If None, creates
                        scenesplit_output in current directory.
            source_video: Source video path for naming context.
        """
        if output_dir is None:
            self.output_dir = Path.cwd() / DEFAULT_OUTPUT_DIR
        else:
            self.output_dir = Path(output_dir)

        self.source_video = source_video

    def prepare(self) -> Path:
        """Create the output directory if it doesn't exist.

        Returns:
            The output directory path.

        Raises:
            OutputError: If directory cannot be created.
        """
        try:
            self.output_dir.mkdir(parents=True, exist_ok=True)
            return self.output_dir
        except OSError as e:
            raise OutputError(f"Failed to create output directory: {e}") from e

    def write_frame(
        self,
        segment: SemanticSegment,
        frame_number: int,
    ) -> FrameMetadata:
        """Write a frame image to disk.

        Args:
            segment: Segment containing the frame to write.
            frame_number: Sequential number for the output filename (1-indexed).

        Returns:
            Metadata about the written frame.

        Raises:
            OutputError: If frame cannot be written.
        """
        filename = f"{frame_number:04d}.{OUTPUT_IMAGE_FORMAT}"
        filepath = self.output_dir / filename

        frame = segment.representative_frame.frame
        try:
            # OpenCV write with quality setting
            encode_params = [cv2.IMWRITE_JPEG_QUALITY, OUTPUT_IMAGE_QUALITY]
            success = cv2.imwrite(str(filepath), frame.data, encode_params)
            if not success:
                raise OutputError(f"Failed to write frame: {filepath}")
        except Exception as e:
            raise OutputError(f"Failed to write frame {filepath}: {e}") from e

        return FrameMetadata(
            filename=filename,
            segment_index=segment.index,
            frame_index=frame.index,
            timestamp_seconds=frame.timestamp_seconds,
            timestamp_formatted=self._format_timestamp(frame.timestamp_seconds),
        )

    def write_frames(
        self,
        segments: Sequence[SemanticSegment],
        progress_callback: callable | None = None,
    ) -> list[FrameMetadata]:
        """Write all segment representative frames to disk.

        Args:
            segments: Segments to write.
            progress_callback: Optional callback(current, total) for progress.

        Returns:
            List of metadata for written frames.
        """
        self.prepare()
        frame_metadata: list[FrameMetadata] = []

        for i, segment in enumerate(segments, start=1):
            metadata = self.write_frame(segment, i)
            frame_metadata.append(metadata)

            if progress_callback is not None:
                progress_callback(i, len(segments))

        return frame_metadata

    def write_metadata(
        self,
        video_metadata: VideoMetadata,
        frame_metadata: list[FrameMetadata],
        detail_level: str,
        quality_preset: str,
    ) -> Path:
        """Write extraction metadata to a JSON file.

        Args:
            video_metadata: Source video metadata.
            frame_metadata: Metadata for extracted frames.
            detail_level: Detail level used for extraction.
            quality_preset: Quality preset used.

        Returns:
            Path to the metadata file.

        Raises:
            OutputError: If metadata cannot be written.
        """
        output_meta = OutputMetadata(
            source_video=str(video_metadata.path),
            video_duration_seconds=video_metadata.duration_seconds,
            video_frame_count=video_metadata.frame_count,
            extracted_frames=len(frame_metadata),
            detail_level=detail_level,
            quality_preset=quality_preset,
            frames=frame_metadata,
        )

        metadata_path = self.output_dir / "metadata.json"
        try:
            with open(metadata_path, "w", encoding="utf-8") as f:
                json.dump(
                    self._to_dict(output_meta),
                    f,
                    indent=2,
                    ensure_ascii=False,
                )
            return metadata_path
        except Exception as e:
            raise OutputError(f"Failed to write metadata: {e}") from e

    def _to_dict(self, obj: Any) -> dict:
        """Convert a dataclass to a dictionary, handling nested dataclasses."""
        if hasattr(obj, "__dataclass_fields__"):
            return {k: self._to_dict(v) for k, v in asdict(obj).items()}
        elif isinstance(obj, list):
            return [self._to_dict(item) for item in obj]
        return obj

    @staticmethod
    def _format_timestamp(seconds: float) -> str:
        """Format a timestamp as HH:MM:SS.mmm."""
        hours = int(seconds // 3600)
        minutes = int((seconds % 3600) // 60)
        secs = seconds % 60
        return f"{hours:02d}:{minutes:02d}:{secs:06.3f}"
