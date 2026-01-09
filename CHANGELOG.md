# Changelog

All notable changes to SceneSplit will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.2] - 2026-01-08

### Fixed

- Fix cross-platform OpenCV compatibility (use `cvt_color_def` instead of `cvt_color` with `AlgorithmHint`)

## [1.1.1] - 2026-01-08

### Fixed

- Add missing LICENSE file for crates.io publishing

## [1.1.0] - 2026-01-07

### Added

- Metadata output: JSON sidecar files with timestamp, segment index, and frame number
- Packaging improvements for crates.io distribution
- GitHub Actions CI/CD pipeline
  - Automated testing on push/PR
  - Multi-platform release builds (macOS ARM64, macOS x64, Linux x64)
  - Automated GitHub releases on tag push
  - Automated crates.io publishing

### Fixed

- Resolved Clippy warnings for cleaner CI builds
- Fixed rustfmt formatting issues
- Added llvm dependency for opencv-rs compilation in CI

## [1.0.0] - 2026-01-01

### Added

- Initial MVP release
- Video ingestion and offline decoding (no network access required)
- Semantic embedding using MobileNetV3 ONNX model
- Cosine similarity-based scene change detection
- Deterministic frame selection (one representative frame per segment)
- CLI interface with two primary controls:
  - `--detail` flag: `key` (default), `summary`, or `all`
  - `--quality` flag: `fast`, `balanced` (default), or `best`
- Auto-download of ONNX model on first run
- Support for common video formats via OpenCV
- PNG output with timeline-ordered naming

### Technical Details

- Built with Rust for performance and safety
- Uses ort (ONNX Runtime) for ML inference
- OpenCV for video decoding
- Fully offline operation after initial model download

[1.1.2]: https://github.com/wilmoore/scenesplit/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/wilmoore/scenesplit/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/wilmoore/scenesplit/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/wilmoore/scenesplit/releases/tag/v1.0.0
