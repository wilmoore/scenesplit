"""Tests for video loading module."""

import tempfile
from pathlib import Path

import cv2
import numpy as np
import pytest

from scenesplit.constants import QualityPreset
from scenesplit.exceptions import (
    UnsupportedFormatError,
    VideoNotFoundError,
)
from scenesplit.video import VideoLoader


def create_test_video(path: Path, width: int = 320, height: int = 240, frames: int = 30) -> None:
    """Create a simple test video file."""
    fourcc = cv2.VideoWriter_fourcc(*"mp4v")
    out = cv2.VideoWriter(str(path), fourcc, 30.0, (width, height))

    for i in range(frames):
        # Create a frame with varying colors
        frame = np.zeros((height, width, 3), dtype=np.uint8)
        frame[:, :, 0] = i * 8 % 256  # Blue channel varies
        frame[:, :, 1] = 128  # Green constant
        frame[:, :, 2] = (255 - i * 8) % 256  # Red channel varies inversely
        out.write(frame)

    out.release()


class TestVideoLoader:
    """Tests for VideoLoader class."""

    def test_load_valid_video(self, tmp_path: Path) -> None:
        """Test loading a valid video file."""
        video_path = tmp_path / "test.mp4"
        create_test_video(video_path, frames=30)

        loader = VideoLoader(video_path)
        metadata = loader.metadata

        assert metadata.width == 320
        assert metadata.height == 240
        assert metadata.frame_count == 30
        assert metadata.fps > 0

    def test_file_not_found(self) -> None:
        """Test error when video file doesn't exist."""
        with pytest.raises(VideoNotFoundError) as exc_info:
            VideoLoader("/nonexistent/video.mp4")

        assert "not found" in str(exc_info.value).lower()

    def test_unsupported_format(self, tmp_path: Path) -> None:
        """Test error for unsupported video format."""
        # Create a file with unsupported extension
        invalid_file = tmp_path / "video.xyz"
        invalid_file.write_text("not a video")

        with pytest.raises(UnsupportedFormatError) as exc_info:
            VideoLoader(invalid_file)

        assert "xyz" in str(exc_info.value).lower()
        assert "supported" in str(exc_info.value).lower()

    def test_extract_frames(self, tmp_path: Path) -> None:
        """Test frame extraction."""
        video_path = tmp_path / "test.mp4"
        create_test_video(video_path, frames=30)

        loader = VideoLoader(video_path)
        frames = list(loader.extract_frames(quality=QualityPreset.FAST))

        # With sample rate of 15, we should get 2 frames from 30
        assert len(frames) == 2
        assert frames[0].index == 0
        assert frames[1].index == 15

    def test_frame_data_shape(self, tmp_path: Path) -> None:
        """Test that extracted frames have correct dimensions."""
        video_path = tmp_path / "test.mp4"
        create_test_video(video_path, width=320, height=240, frames=10)

        loader = VideoLoader(video_path)
        frames = list(loader.extract_frames(quality=QualityPreset.BEST))

        for frame in frames:
            assert frame.data.shape == (240, 320, 3)

    def test_get_frame_at(self, tmp_path: Path) -> None:
        """Test getting a specific frame by index."""
        video_path = tmp_path / "test.mp4"
        create_test_video(video_path, frames=30)

        loader = VideoLoader(video_path)
        frame = loader.get_frame_at(15)

        assert frame.index == 15
        assert frame.data.shape == (240, 320, 3)

    def test_get_frame_at_out_of_range(self, tmp_path: Path) -> None:
        """Test error when frame index is out of range."""
        video_path = tmp_path / "test.mp4"
        create_test_video(video_path, frames=30)

        loader = VideoLoader(video_path)

        with pytest.raises(IndexError):
            loader.get_frame_at(100)

    def test_context_manager(self, tmp_path: Path) -> None:
        """Test using VideoLoader as context manager."""
        video_path = tmp_path / "test.mp4"
        create_test_video(video_path, frames=10)

        with VideoLoader(video_path) as loader:
            metadata = loader.metadata
            assert metadata.frame_count == 10
