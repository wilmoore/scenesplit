"""Tests for semantic segmentation module."""

import numpy as np
import pytest

from scenesplit.constants import DetailLevel
from scenesplit.embeddings import EmbeddedFrame, cosine_similarity
from scenesplit.segmentation import SemanticSegmenter, deterministic_frame_selection
from scenesplit.video import Frame


def create_mock_frame(index: int, timestamp: float) -> Frame:
    """Create a mock Frame object."""
    return Frame(
        index=index,
        timestamp_seconds=timestamp,
        data=np.zeros((100, 100, 3), dtype=np.uint8),
    )


def create_embedded_frame(
    index: int,
    timestamp: float,
    embedding: np.ndarray,
) -> EmbeddedFrame:
    """Create an EmbeddedFrame with the given embedding."""
    frame = create_mock_frame(index, timestamp)
    # Normalize embedding
    normalized = embedding / np.linalg.norm(embedding)
    return EmbeddedFrame(frame=frame, embedding=normalized.astype(np.float32))


class TestCosineSimlarity:
    """Tests for cosine similarity function."""

    def test_identical_vectors(self) -> None:
        """Test that identical vectors have similarity 1.0."""
        v = np.array([1.0, 0.0, 0.0], dtype=np.float32)
        assert cosine_similarity(v, v) == pytest.approx(1.0)

    def test_orthogonal_vectors(self) -> None:
        """Test that orthogonal vectors have similarity 0.0."""
        v1 = np.array([1.0, 0.0, 0.0], dtype=np.float32)
        v2 = np.array([0.0, 1.0, 0.0], dtype=np.float32)
        assert cosine_similarity(v1, v2) == pytest.approx(0.0)

    def test_opposite_vectors(self) -> None:
        """Test that opposite vectors have similarity -1.0."""
        v1 = np.array([1.0, 0.0, 0.0], dtype=np.float32)
        v2 = np.array([-1.0, 0.0, 0.0], dtype=np.float32)
        assert cosine_similarity(v1, v2) == pytest.approx(-1.0)


class TestSemanticSegmenter:
    """Tests for SemanticSegmenter class."""

    def test_empty_input(self) -> None:
        """Test segmentation of empty frame list."""
        segmenter = SemanticSegmenter(detail=DetailLevel.SUMMARY)
        segments = segmenter.segment([])
        assert segments == []

    def test_single_frame(self) -> None:
        """Test segmentation with a single frame."""
        embedding = np.random.randn(2048).astype(np.float32)
        frames = [create_embedded_frame(0, 0.0, embedding)]

        segmenter = SemanticSegmenter(detail=DetailLevel.SUMMARY)
        segments = segmenter.segment(frames)

        assert len(segments) == 1
        assert segments[0].frame_count == 1

    def test_similar_frames_grouped(self) -> None:
        """Test that similar frames are grouped together."""
        # Create frames with very similar embeddings
        base_embedding = np.random.randn(2048).astype(np.float32)
        frames = []
        for i in range(100):
            # Add small noise to keep embeddings similar
            noisy = base_embedding + np.random.randn(2048).astype(np.float32) * 0.01
            frames.append(create_embedded_frame(i, i / 30.0, noisy))

        segmenter = SemanticSegmenter(detail=DetailLevel.SUMMARY)
        segments = segmenter.segment(frames)

        # Similar frames should result in few segments
        assert len(segments) <= 3

    def test_different_frames_separated(self) -> None:
        """Test that very different frames create new segments."""
        frames = []

        # First group of similar frames
        base1 = np.random.randn(2048).astype(np.float32)
        for i in range(50):
            noisy = base1 + np.random.randn(2048).astype(np.float32) * 0.01
            frames.append(create_embedded_frame(i, i / 30.0, noisy))

        # Second group with very different embedding
        base2 = np.random.randn(2048).astype(np.float32)
        for i in range(50, 100):
            noisy = base2 + np.random.randn(2048).astype(np.float32) * 0.01
            frames.append(create_embedded_frame(i, i / 30.0, noisy))

        segmenter = SemanticSegmenter(detail=DetailLevel.SUMMARY)
        segments = segmenter.segment(frames)

        # Should have at least 2 segments (one for each group)
        assert len(segments) >= 2

    def test_segments_ordered_by_index(self) -> None:
        """Test that segments are ordered by their index."""
        frames = []
        for i in range(100):
            # Create varying embeddings
            embedding = np.random.randn(2048).astype(np.float32)
            frames.append(create_embedded_frame(i, i / 30.0, embedding))

        segmenter = SemanticSegmenter(detail=DetailLevel.ALL)
        segments = segmenter.segment(frames)

        for i in range(1, len(segments)):
            assert segments[i].index > segments[i - 1].index

    def test_detail_level_affects_segment_count(self) -> None:
        """Test that detail level affects the number of segments."""
        frames = []
        for i in range(200):
            embedding = np.random.randn(2048).astype(np.float32)
            # Add structure: change base embedding every 40 frames
            base_idx = i // 40
            base = np.zeros(2048)
            base[base_idx * 100:(base_idx + 1) * 100] = 1.0
            embedding = embedding * 0.3 + base * 0.7
            frames.append(create_embedded_frame(i, i / 30.0, embedding))

        key_segments = SemanticSegmenter(detail=DetailLevel.KEY).segment(frames)
        all_segments = SemanticSegmenter(detail=DetailLevel.ALL).segment(frames)

        # 'all' should produce more segments than 'key'
        assert len(all_segments) >= len(key_segments)


class TestDeterministicFrameSelection:
    """Tests for deterministic frame selection."""

    def test_returns_representative_frames(self) -> None:
        """Test that selection returns representative frames from segments."""
        frames = []
        for i in range(100):
            embedding = np.random.randn(2048).astype(np.float32)
            frames.append(create_embedded_frame(i, i / 30.0, embedding))

        segmenter = SemanticSegmenter(detail=DetailLevel.SUMMARY)
        segments = segmenter.segment(frames)

        selected = deterministic_frame_selection(segments)

        assert len(selected) == len(segments)
        for i, frame in enumerate(selected):
            assert frame == segments[i].representative_frame

    def test_deterministic_ordering(self) -> None:
        """Test that selection is deterministic and ordered."""
        frames = []
        for i in range(100):
            embedding = np.random.randn(2048).astype(np.float32)
            frames.append(create_embedded_frame(i, i / 30.0, embedding))

        segmenter = SemanticSegmenter(detail=DetailLevel.SUMMARY)
        segments = segmenter.segment(frames)

        # Run selection twice
        selected1 = deterministic_frame_selection(segments)
        selected2 = deterministic_frame_selection(segments)

        # Should be identical
        assert len(selected1) == len(selected2)
        for f1, f2 in zip(selected1, selected2):
            assert f1.index == f2.index
