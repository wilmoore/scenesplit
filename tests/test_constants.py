"""Tests for constants and configuration."""

import pytest

from scenesplit.constants import DetailLevel, QualityPreset


class TestDetailLevel:
    """Tests for DetailLevel enum."""

    def test_values(self) -> None:
        """Test that all expected values exist."""
        assert DetailLevel.KEY.value == "key"
        assert DetailLevel.SUMMARY.value == "summary"
        assert DetailLevel.ALL.value == "all"

    def test_similarity_thresholds_ordering(self) -> None:
        """Test that thresholds are ordered correctly (key > summary > all)."""
        # Higher threshold = fewer frames
        assert DetailLevel.KEY.similarity_threshold > DetailLevel.SUMMARY.similarity_threshold
        assert DetailLevel.SUMMARY.similarity_threshold > DetailLevel.ALL.similarity_threshold

    def test_min_segment_frames_ordering(self) -> None:
        """Test that minimum segment frames are ordered correctly."""
        # Key should have the largest minimum (fewest, more stable segments)
        assert DetailLevel.KEY.min_segment_frames > DetailLevel.SUMMARY.min_segment_frames
        assert DetailLevel.SUMMARY.min_segment_frames > DetailLevel.ALL.min_segment_frames


class TestQualityPreset:
    """Tests for QualityPreset enum."""

    def test_values(self) -> None:
        """Test that all expected values exist."""
        assert QualityPreset.FAST.value == "fast"
        assert QualityPreset.BALANCED.value == "balanced"
        assert QualityPreset.BEST.value == "best"

    def test_sample_rate_ordering(self) -> None:
        """Test that sample rates are ordered correctly (fast > balanced > best)."""
        # Higher sample rate = fewer frames processed = faster
        assert QualityPreset.FAST.frame_sample_rate > QualityPreset.BALANCED.frame_sample_rate
        assert QualityPreset.BALANCED.frame_sample_rate > QualityPreset.BEST.frame_sample_rate

    def test_best_processes_all_frames(self) -> None:
        """Test that best quality processes every frame."""
        assert QualityPreset.BEST.frame_sample_rate == 1

    def test_resize_factor_ordering(self) -> None:
        """Test that resize factors are ordered correctly."""
        # Higher factor = larger images = slower but better
        assert QualityPreset.BEST.image_resize_factor >= QualityPreset.BALANCED.image_resize_factor
        assert QualityPreset.BALANCED.image_resize_factor >= QualityPreset.FAST.image_resize_factor

    def test_best_uses_full_resolution(self) -> None:
        """Test that best quality uses full resolution."""
        assert QualityPreset.BEST.image_resize_factor == 1.0
