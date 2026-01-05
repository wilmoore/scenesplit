# 003. Auto-Download Model on First Run

Date: 2025-01-04

## Status

Accepted

## Context

The PRD specifies "offline-first" operation, but also notes that users shouldn't need to manually download ML models. ResNet50 ONNX model is ~100MB.

Options:
1. Embed model in binary (would make binary ~100MB+)
2. Require user to download model manually
3. Auto-download on first run and cache locally

## Decision

Auto-download the model from ONNX Model Zoo on first run:
- Download URL: `https://github.com/onnx/models/raw/main/validated/vision/classification/resnet/model/resnet50-v2-7.onnx`
- Cache location: `~/.cache/scenesplit/resnet50-v2-7.onnx`
- Progress bar shown during download
- Subsequent runs use cached model (no network needed)

Users can override with `--model` flag to use custom models.

## Consequences

**Positive:**
- Zero-config first run experience
- Small binary size (2.4MB vs 100MB+)
- Offline operation after first run
- Users can still provide custom models

**Negative:**
- First run requires network access
- Download can fail (handled with clear error messages)
- Cache location varies by platform

## Alternatives Considered

- **Embedded model:** Rejected due to binary size bloat
- **Required manual download:** Rejected as poor UX
- **Separate installer:** Overkill for single model file

## Related

- Planning: `.plan/.done/mvp-scenesplit/`
- Implementation: `src/model.rs`
