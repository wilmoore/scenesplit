# 001. Use Rust for Implementation

Date: 2025-01-04

## Status

Accepted

## Context

SceneSplit needs to be an offline CLI tool that processes video files and extracts semantically distinct frames. The PRD emphasizes single-binary distribution and lean dependency footprint.

Initial implementation was done in Python, but this required users to manage Python environments, install PyTorch and other heavy ML dependencies.

## Decision

Rewrite the implementation in Rust, using:
- `opencv` crate for video decoding
- `ort` crate for ONNX Runtime inference
- `clap` crate for CLI parsing

## Consequences

**Positive:**
- Single binary distribution (~2.4MB optimized)
- No Python/pip dependency management for users
- Better startup time and memory efficiency
- Cross-compilation possible for multiple platforms

**Negative:**
- Smaller ecosystem for ML compared to Python
- OpenCV and ONNX Runtime still required as system dependencies
- More complex build setup (libclang, library paths)

## Alternatives Considered

- **Python + PyTorch:** Rejected due to heavy dependencies and complex distribution
- **Go:** Considered but Rust has better ONNX Runtime bindings and performance characteristics

## Related

- Planning: `.plan/.done/mvp-scenesplit/`
