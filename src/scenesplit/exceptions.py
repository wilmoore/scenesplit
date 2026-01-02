"""Custom exceptions for SceneSplit."""


class SceneSplitError(Exception):
    """Base exception for SceneSplit errors."""

    pass


class VideoLoadError(SceneSplitError):
    """Error loading or decoding a video file."""

    pass


class UnsupportedFormatError(VideoLoadError):
    """Video format is not supported."""

    def __init__(self, path: str, extension: str) -> None:
        self.path = path
        self.extension = extension
        super().__init__(
            f"Unsupported video format: '{extension}'. "
            f"Supported formats: mp4, avi, mov, mkv, webm, m4v, flv, wmv, mpeg, mpg"
        )


class VideoNotFoundError(VideoLoadError):
    """Video file does not exist."""

    def __init__(self, path: str) -> None:
        self.path = path
        super().__init__(f"Video file not found: {path}")


class VideoDecodeError(VideoLoadError):
    """Error decoding video frames."""

    def __init__(self, path: str, reason: str) -> None:
        self.path = path
        self.reason = reason
        super().__init__(f"Failed to decode video '{path}': {reason}")


class EmbeddingError(SceneSplitError):
    """Error computing embeddings."""

    pass


class OutputError(SceneSplitError):
    """Error writing output files."""

    pass
