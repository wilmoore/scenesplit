# SceneSplit

Extract semantically distinct still images from video.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-2021-orange.svg)
![Version](https://img.shields.io/badge/version-1.0.0-green.svg)

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

```bash
git clone https://github.com/your-org/scenesplit.git
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
├── frame_001.png
├── frame_002.png
├── frame_003.png
├── ...
└── metadata.json
```

### metadata.json

```json
{
  "source_video": "video.mp4",
  "detail_level": "summary",
  "quality_preset": "balanced",
  "frames": [
    {
      "filename": "frame_001.png",
      "frame_index": 0,
      "timestamp_seconds": 0.0
    },
    {
      "filename": "frame_002.png",
      "frame_index": 45,
      "timestamp_seconds": 1.5
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
