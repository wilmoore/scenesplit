# 002. Use ONNX Runtime for ML Inference

Date: 2025-01-04

## Status

Accepted

## Context

Semantic keyframe extraction requires computing vision embeddings for video frames. This needs an ML inference runtime that can run ResNet50 or similar models.

Options considered:
- Native Rust ML frameworks (burn, candle)
- ONNX Runtime via ort crate
- TensorFlow Lite

## Decision

Use ONNX Runtime via the `ort` crate with `load-dynamic` feature, loading `libonnxruntime.dylib` at runtime.

ResNet50-v2-7 from ONNX Model Zoo serves as the default embedding model.

## Consequences

**Positive:**
- Mature, production-tested inference engine
- Broad model compatibility (any ONNX model works)
- Good performance with hardware acceleration
- User can provide custom ONNX models via `--model` flag

**Negative:**
- Requires ONNX Runtime installed on system (via Homebrew on macOS)
- Dynamic loading requires rpath configuration for library discovery
- Large model file (~100MB) needs downloading on first run

## Alternatives Considered

- **burn/candle:** Native Rust but limited model support and less mature
- **TensorFlow Lite:** Would require additional bindings and model conversion

## Related

- Planning: `.plan/.done/mvp-scenesplit/`
