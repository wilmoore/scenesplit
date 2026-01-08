# 004. Use GitHub Actions for Releases

Date: 2026-01-07

## Status

Accepted

## Context

SceneSplit is a Rust CLI tool that depends on native libraries (OpenCV, ONNX Runtime). Users need pre-built binaries for easy installation without requiring a full development environment with Rust, OpenCV, and ONNX Runtime.

We needed a solution for:
- Automated cross-platform builds on tag push
- Binary distribution for macOS (ARM64 and Intel) and Linux (x64)
- CI for pull request validation

## Decision

Use GitHub Actions with:
- **Release workflow** triggered on version tags (`v*`) that builds for all platforms using a matrix strategy
- **CI workflow** for PRs with format check, clippy, build, and test
- Pre-built binaries distributed as GitHub Release assets (`.tar.gz` archives)

## Consequences

### Positive
- Zero manual effort for releases after tagging
- Users get one-liner installation via `curl | tar`
- Native builds on each platform ensure compatibility
- CI catches issues before merge

### Negative
- Build times depend on GitHub-hosted runner availability
- OpenCV and ONNX Runtime must be installed on CI runners (added complexity)
- Binary size may be larger than system-linked builds

## Alternatives Considered

1. **Cross-compilation from single machine** - Rejected due to native library linking complexity
2. **Docker-based builds** - Rejected due to added complexity for a CLI tool
3. **Manual releases** - Rejected due to human error risk and friction

## Related

- Planning: `.plan/.done/feat-metadata-and-packaging/`
