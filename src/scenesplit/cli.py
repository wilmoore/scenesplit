"""Command-line interface for SceneSplit."""

import sys
from pathlib import Path

import click

from scenesplit import __version__
from scenesplit.constants import DetailLevel, QualityPreset
from scenesplit.exceptions import SceneSplitError
from scenesplit.processor import SceneSplitProcessor


def _progress_callback(stage: str, current: int, total: int) -> None:
    """Print progress information."""
    if total > 0:
        click.echo(f"{stage}... ({current}/{total})")
    else:
        click.echo(f"{stage}...")


@click.command()
@click.argument(
    "input_video",
    type=click.Path(exists=True, dir_okay=False, path_type=Path),
)
@click.option(
    "--detail",
    type=click.Choice([d.value for d in DetailLevel], case_sensitive=False),
    default=DetailLevel.SUMMARY.value,
    help="Granularity level: 'key' (minimal), 'summary' (moderate), 'all' (comprehensive)",
)
@click.option(
    "--quality",
    type=click.Choice([q.value for q in QualityPreset], case_sensitive=False),
    default=QualityPreset.BALANCED.value,
    help="Processing quality: 'fast', 'balanced', or 'best'",
)
@click.option(
    "--output", "-o",
    type=click.Path(file_okay=False, path_type=Path),
    default=None,
    help="Output directory (default: ./scenesplit_output/)",
)
@click.option(
    "--quiet", "-q",
    is_flag=True,
    default=False,
    help="Suppress progress output",
)
@click.version_option(version=__version__)
def main(
    input_video: Path,
    detail: str,
    quality: str,
    output: Path | None,
    quiet: bool,
) -> None:
    """Extract semantically distinct still images from a video.

    SceneSplit analyzes VIDEO_FILE and extracts representative frames that
    capture meaningful visual changes. Output is written to a directory
    containing numbered images and a metadata.json file.

    \b
    Examples:
        scenesplit video.mp4
        scenesplit video.mp4 --detail key --quality fast
        scenesplit video.mp4 --detail all --output ./my_output/
    """
    try:
        # Convert string options to enums
        detail_level = DetailLevel(detail)
        quality_preset = QualityPreset(quality)

        if not quiet:
            click.echo(f"SceneSplit v{__version__}")
            click.echo(f"Input: {input_video}")
            click.echo(f"Detail: {detail_level.value}")
            click.echo(f"Quality: {quality_preset.value}")
            click.echo()

        # Create processor and run
        processor = SceneSplitProcessor(
            detail=detail_level,
            quality=quality_preset,
            output_dir=output,
        )

        progress_cb = None if quiet else _progress_callback
        result = processor.process(input_video, progress_callback=progress_cb)

        # Print summary
        if not quiet:
            click.echo()
            click.echo("=" * 50)
            click.echo(f"Extracted {result.frames_extracted} stills")
            click.echo(f"Output written to {result.output_dir}/")
            click.echo("=" * 50)

    except SceneSplitError as e:
        click.echo(f"Error: {e}", err=True)
        sys.exit(1)
    except Exception as e:
        click.echo(f"Unexpected error: {e}", err=True)
        sys.exit(1)


if __name__ == "__main__":
    main()
