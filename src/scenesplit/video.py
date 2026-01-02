"""Video loading and frame extraction module."""

from dataclasses import dataclass
from pathlib import Path
from typing import Iterator

import cv2
import numpy as np
from numpy.typing import NDArray

from scenesplit.constants import SUPPORTED_VIDEO_FORMATS, QualityPreset
from scenesplit.exceptions import (
    UnsupportedFormatError,
    VideoDecodeError,
    VideoNotFoundError,
)


@dataclass(frozen=True)
class VideoMetadata:
    """Metadata extracted from a video file."""

    path: Path
    width: int
    height: int
    fps: float
    frame_count: int
    duration_seconds: float
    codec: str


@dataclass(frozen=True)
class Frame:
    """A single video frame with metadata."""

    index: int
    timestamp_seconds: float
    data: NDArray[np.uint8]

    @property
    def timestamp_ms(self) -> int:
        """Timestamp in milliseconds."""
        return int(self.timestamp_seconds * 1000)


class VideoLoader:
    """Load and extract frames from video files.

    This class handles video file validation, opening, and frame extraction.
    All processing is done offline without any network access.
    """

    def __init__(self, path: str | Path) -> None:
        """Initialize the video loader.

        Args:
            path: Path to the video file.

        Raises:
            VideoNotFoundError: If the file does not exist.
            UnsupportedFormatError: If the file format is not supported.
            VideoDecodeError: If the video cannot be opened or decoded.
        """
        self.path = Path(path).resolve()
        self._validate_file()
        self._metadata: VideoMetadata | None = None
        self._cap: cv2.VideoCapture | None = None

    def _validate_file(self) -> None:
        """Validate the video file exists and has a supported format."""
        if not self.path.exists():
            raise VideoNotFoundError(str(self.path))

        if not self.path.is_file():
            raise VideoNotFoundError(f"Path is not a file: {self.path}")

        extension = self.path.suffix.lower().lstrip(".")
        if extension not in SUPPORTED_VIDEO_FORMATS:
            raise UnsupportedFormatError(str(self.path), extension)

    def _open(self) -> cv2.VideoCapture:
        """Open the video file and return the capture object."""
        cap = cv2.VideoCapture(str(self.path))
        if not cap.isOpened():
            raise VideoDecodeError(str(self.path), "Failed to open video file")
        return cap

    @property
    def metadata(self) -> VideoMetadata:
        """Get video metadata, loading it if necessary."""
        if self._metadata is None:
            cap = self._open()
            try:
                width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
                height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
                fps = cap.get(cv2.CAP_PROP_FPS)
                frame_count = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
                fourcc = int(cap.get(cv2.CAP_PROP_FOURCC))

                # Decode fourcc to string
                codec = "".join([chr((fourcc >> 8 * i) & 0xFF) for i in range(4)])

                # Calculate duration
                duration = frame_count / fps if fps > 0 else 0.0

                self._metadata = VideoMetadata(
                    path=self.path,
                    width=width,
                    height=height,
                    fps=fps,
                    frame_count=frame_count,
                    duration_seconds=duration,
                    codec=codec.strip(),
                )
            finally:
                cap.release()

        return self._metadata

    def extract_frames(
        self,
        quality: QualityPreset = QualityPreset.BALANCED,
        progress_callback: callable | None = None,
    ) -> Iterator[Frame]:
        """Extract frames from the video at the specified sample rate.

        Args:
            quality: Quality preset determining sample rate.
            progress_callback: Optional callback(current, total) for progress.

        Yields:
            Frame objects containing frame data and metadata.

        Raises:
            VideoDecodeError: If frames cannot be read.
        """
        cap = self._open()
        try:
            sample_rate = quality.frame_sample_rate
            metadata = self.metadata
            total_frames = metadata.frame_count
            fps = metadata.fps

            frame_index = 0
            sampled_count = 0

            while True:
                ret, frame_data = cap.read()
                if not ret:
                    break

                # Only yield frames at the sample rate
                if frame_index % sample_rate == 0:
                    timestamp = frame_index / fps if fps > 0 else 0.0
                    yield Frame(
                        index=frame_index,
                        timestamp_seconds=timestamp,
                        data=frame_data,
                    )
                    sampled_count += 1

                frame_index += 1

                if progress_callback is not None:
                    progress_callback(frame_index, total_frames)

        finally:
            cap.release()

    def get_frame_at(self, index: int) -> Frame:
        """Get a specific frame by index.

        Args:
            index: Frame index (0-based).

        Returns:
            The Frame at the specified index.

        Raises:
            VideoDecodeError: If the frame cannot be read.
            IndexError: If the index is out of range.
        """
        metadata = self.metadata
        if index < 0 or index >= metadata.frame_count:
            raise IndexError(
                f"Frame index {index} out of range [0, {metadata.frame_count})"
            )

        cap = self._open()
        try:
            cap.set(cv2.CAP_PROP_POS_FRAMES, index)
            ret, frame_data = cap.read()
            if not ret:
                raise VideoDecodeError(
                    str(self.path), f"Failed to read frame at index {index}"
                )

            timestamp = index / metadata.fps if metadata.fps > 0 else 0.0
            return Frame(
                index=index,
                timestamp_seconds=timestamp,
                data=frame_data,
            )
        finally:
            cap.release()

    def __enter__(self) -> "VideoLoader":
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """Context manager exit."""
        if self._cap is not None:
            self._cap.release()
            self._cap = None
