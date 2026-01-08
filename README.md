# SceneSplit

Extract semantically distinct still images from video.

[![CI](https://github.com/wilmoore/scenesplit/actions/workflows/ci.yml/badge.svg)](https://github.com/wilmoore/scenesplit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/scenesplit.svg)](https://crates.io/crates/scenesplit)
[![License](https://img.shields.io/crates/l/scenesplit.svg)](https://github.com/wilmoore/scenesplit/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)

---

SceneSplit analyzes video files and extracts representative frames that capture meaningful visual changes. Unlike simple keyframe extraction that relies on codec markers, SceneSplit uses vision embeddings to detect semantic transitions—scene changes, subject shifts, and visual content boundaries.

## Features

- **Semantic analysis** — Uses ResNet50 embeddings to detect meaningful visual changes, not just codec keyframes
- **Offline processing** — All analysis runs locally after initial model download
- **Two-knob configuration** — Control output granularity (`--detail`) and processing fidelity (`--quality`)
- **Auto-download model** — ResNet50 ONNX model (~100MB) downloads automatically on first run
- **Structured output** — Numbered images plus `metadata.json` with timestamps and frame indices

## Prerequisites

- **Rust 1.70+** — [Install Rust](https://rustup.rs/)
- **OpenCV 4.x** — Required for video decoding
- **ONNX Runtime** — Required for ML inference

### macOS (Homebrew)

```bash
brew install opencv onnxruntime
```

### Ubuntu/Debian

```bash
sudo apt install libopencv-dev
# ONNX Runtime: download from https://github.com/microsoft/onnxruntime/releases
```

## Installation

### From crates.io

```bash
cargo install scenesplit
```

### Pre-built Binaries

Download the latest release for your platform from [Releases](https://github.com/wilmoore/scenesplit/releases):

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `scenesplit-macos-arm64.tar.gz` |
| macOS (Intel) | `scenesplit-macos-x64.tar.gz` |
| Linux (x64) | `scenesplit-linux-x64.tar.gz` |

```bash
# macOS (Apple Silicon)
curl -L https://github.com/wilmoore/scenesplit/releases/latest/download/scenesplit-macos-arm64.tar.gz | tar xz
sudo mv scenesplit /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/wilmoore/scenesplit/releases/latest/download/scenesplit-macos-x64.tar.gz | tar xz
sudo mv scenesplit /usr/local/bin/

# Linux
curl -L https://github.com/wilmoore/scenesplit/releases/latest/download/scenesplit-linux-x64.tar.gz | tar xz
sudo mv scenesplit /usr/local/bin/
```

### Build from Source

```bash
git clone https://github.com/wilmoore/scenesplit.git
cd scenesplit
cargo build --release
```

The binary is at `target/release/scenesplit`.

## Quick Start

```bash
# Extract frames with default settings
scenesplit video.mp4

# Minimal output (5-10 frames)
scenesplit -d key video.mp4

# Comprehensive extraction with highest quality
scenesplit -d all -q best -o ./frames video.mp4
```

## Usage

```
scenesplit [OPTIONS] <VIDEO>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<VIDEO>` | Path to the input video file |

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `-d, --detail <LEVEL>` | `summary` | Granularity level |
| `-q, --quality <PRESET>` | `balanced` | Processing quality |
| `-m, --model <PATH>` | auto-download | Custom ONNX model file |
| `-o, --output <DIR>` | `./scenesplit_output/` | Output directory |
| `-s, --quiet` | off | Suppress progress output |

### Detail Levels

| Level | Frames | Use Case |
|-------|--------|----------|
| `key` | 5-10 | Major scene changes only |
| `summary` | 10-20 | Representative frames |
| `all` | 20-30 | Comprehensive semantic changes |

### Quality Presets

| Preset | Speed | Fidelity |
|--------|-------|----------|
| `fast` | Fastest | Lower |
| `balanced` | Moderate | Good |
| `best` | Slowest | Highest |

## Output

SceneSplit creates a directory containing:

```
scenesplit_output/
├── 0001.jpg
├── 0002.jpg
├── 0003.jpg
├── ...
└── metadata.json
```

### metadata.json

```json
{
  "source_video": "video.mp4",
  "video_duration_seconds": 120.5,
  "video_frame_count": 3615,
  "extracted_frames": 12,
  "detail_level": "summary",
  "quality_preset": "balanced",
  "frames": [
    {
      "filename": "0001.jpg",
      "segment_index": 0,
      "frame_index": 0,
      "timestamp_seconds": 0.0,
      "timestamp_formatted": "00:00:00.000"
    },
    {
      "filename": "0002.jpg",
      "segment_index": 1,
      "frame_index": 45,
      "timestamp_seconds": 1.5,
      "timestamp_formatted": "00:00:01.500"
    }
  ]
}
```

## Supported Formats

- MP4
- AVI
- MOV
- MKV
- WebM

Format support depends on your OpenCV build and available codecs.

## How It Works

1. **Frame extraction** — Samples frames from video at a rate determined by quality preset
2. **Embedding computation** — Passes frames through ResNet50 to generate semantic feature vectors
3. **Similarity analysis** — Computes cosine similarity between consecutive embeddings
4. **Segmentation** — Detects boundaries where similarity drops below threshold (controlled by detail level)
5. **Selection** — Chooses representative frames from each segment

## License

MIT
