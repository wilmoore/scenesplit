"""Tests for CLI module."""

from pathlib import Path

import cv2
import numpy as np
from click.testing import CliRunner

from scenesplit.cli import main


def create_test_video(path: Path, width: int = 160, height: int = 120, frames: int = 30) -> None:
    """Create a simple test video file."""
    fourcc = cv2.VideoWriter_fourcc(*"mp4v")
    out = cv2.VideoWriter(str(path), fourcc, 30.0, (width, height))

    for i in range(frames):
        frame = np.zeros((height, width, 3), dtype=np.uint8)
        # Create distinct scenes every 10 frames
        color_idx = i // 10
        frame[:, :, color_idx % 3] = 200
        out.write(frame)

    out.release()


class TestCLI:
    """Tests for the CLI interface."""

    def test_help(self) -> None:
        """Test --help flag."""
        runner = CliRunner()
        result = runner.invoke(main, ["--help"])

        assert result.exit_code == 0
        assert "Extract semantically distinct" in result.output
        assert "--detail" in result.output
        assert "--quality" in result.output

    def test_version(self) -> None:
        """Test --version flag."""
        runner = CliRunner()
        result = runner.invoke(main, ["--version"])

        assert result.exit_code == 0
        assert "1.0.0" in result.output

    def test_missing_input(self) -> None:
        """Test error when input file is missing."""
        runner = CliRunner()
        result = runner.invoke(main, ["/nonexistent/video.mp4"])

        assert result.exit_code != 0

    def test_detail_options(self) -> None:
        """Test that all detail options are valid."""
        runner = CliRunner()

        # Check help shows all options
        result = runner.invoke(main, ["--help"])
        assert "key" in result.output
        assert "summary" in result.output
        assert "all" in result.output

    def test_quality_options(self) -> None:
        """Test that all quality options are valid."""
        runner = CliRunner()

        result = runner.invoke(main, ["--help"])
        assert "fast" in result.output
        assert "balanced" in result.output
        assert "best" in result.output

    def test_quiet_flag(self) -> None:
        """Test --quiet flag is available."""
        runner = CliRunner()
        result = runner.invoke(main, ["--help"])

        assert "--quiet" in result.output or "-q" in result.output

    def test_output_option(self) -> None:
        """Test --output option is available."""
        runner = CliRunner()
        result = runner.invoke(main, ["--help"])

        assert "--output" in result.output or "-o" in result.output
