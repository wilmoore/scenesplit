"""Constants for SceneSplit configuration."""

from enum import Enum
from typing import Final

# Supported video formats (extensions without dot)
SUPPORTED_VIDEO_FORMATS: Final[frozenset[str]] = frozenset({
    "mp4", "avi", "mov", "mkv", "webm", "m4v", "flv", "wmv", "mpeg", "mpg"
})

# Default output directory name
DEFAULT_OUTPUT_DIR: Final[str] = "scenesplit_output"

# Image output format
OUTPUT_IMAGE_FORMAT: Final[str] = "jpg"
OUTPUT_IMAGE_QUALITY: Final[int] = 95


class DetailLevel(str, Enum):
    """Detail level for frame extraction granularity."""

    KEY = "key"       # Minimal: only major scene changes (5-10 frames for long video)
    SUMMARY = "summary"  # Moderate: representative frames (10-20 frames)
    ALL = "all"       # Comprehensive: all detected semantic changes (20-30 frames)

    @property
    def similarity_threshold(self) -> float:
        """Get cosine similarity threshold for this detail level.

        Lower threshold = more frames (smaller changes detected).
        Higher threshold = fewer frames (only major changes detected).
        """
        thresholds = {
            DetailLevel.KEY: 0.92,      # Very similar frames grouped together
            DetailLevel.SUMMARY: 0.85,  # Moderate grouping
            DetailLevel.ALL: 0.75,      # More sensitive to changes
        }
        return thresholds[self]

    @property
    def min_segment_frames(self) -> int:
        """Minimum frames between keyframes to avoid over-extraction."""
        minimums = {
            DetailLevel.KEY: 90,      # At least 3 seconds at 30fps
            DetailLevel.SUMMARY: 45,  # At least 1.5 seconds
            DetailLevel.ALL: 15,      # At least 0.5 seconds
        }
        return minimums[self]


class QualityPreset(str, Enum):
    """Quality preset affecting processing fidelity and speed."""

    FAST = "fast"         # Quick processing, lower fidelity
    BALANCED = "balanced" # Default: good balance
    BEST = "best"         # Highest quality, slower

    @property
    def frame_sample_rate(self) -> int:
        """Sample every Nth frame for embedding computation.

        Lower = more frames processed = slower but more accurate.
        """
        rates = {
            QualityPreset.FAST: 15,      # Process every 15th frame
            QualityPreset.BALANCED: 5,   # Process every 5th frame
            QualityPreset.BEST: 1,       # Process every frame
        }
        return rates[self]

    @property
    def embedding_batch_size(self) -> int:
        """Batch size for embedding computation."""
        sizes = {
            QualityPreset.FAST: 64,
            QualityPreset.BALANCED: 32,
            QualityPreset.BEST: 16,
        }
        return sizes[self]

    @property
    def image_resize_factor(self) -> float:
        """Factor to resize images for embedding (1.0 = full size)."""
        factors = {
            QualityPreset.FAST: 0.5,
            QualityPreset.BALANCED: 0.75,
            QualityPreset.BEST: 1.0,
        }
        return factors[self]
