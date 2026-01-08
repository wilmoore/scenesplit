# feat/metadata-and-packaging

## Items Addressed

- **#5** - Metadata output (E-005): ✅ Already implemented in `output.rs`
- **#13** - Minimal metadata per image (US-006): ✅ Already implemented
- **#6** - Packaging and distribution (E-006): ✅ GitHub Actions workflows added

## Implementation Details

### Metadata Output

The existing codebase already generates `metadata.json` with:
- `timestamp_seconds` - Frame timestamp in seconds
- `timestamp_formatted` - Human-readable timestamp (HH:MM:SS.mmm)
- `segment_index` - Semantic segment identifier
- `frame_index` - Original frame index from video
- `filename` - Output image filename

This satisfies all acceptance criteria for items #5 and #13.

### Packaging and Distribution

Added GitHub Actions workflows:

1. **`.github/workflows/release.yml`** - Triggered on tag push (`v*`)
   - Builds for macOS ARM64, macOS x64, and Linux x64
   - Creates release with downloadable tarballs
   - Uses matrix strategy for parallel builds

2. **`.github/workflows/ci.yml`** - Triggered on PR/push to main
   - Runs formatting check, clippy, build, and tests

### README Updates

- Added pre-built binary installation instructions
- Updated metadata.json example to match actual output format
- Updated output file naming convention (0001.jpg vs frame_001.png)

## Files Changed

- `.github/workflows/release.yml` (new)
- `.github/workflows/ci.yml` (new)
- `README.md` (updated installation section and output examples)

## Next Steps

1. Commit and push changes
2. Create a tag (e.g., `v1.1.0`) to trigger the release workflow
3. Verify release artifacts on GitHub
