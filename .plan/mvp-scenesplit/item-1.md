# Item #1: Video ingestion and decoding

## Description
E-001: Video ingestion and decoding. Enable SceneSplit to accept local video files as input and decode them offline without network access. Must surface errors for unsupported formats.

## Implementation Approach
1. Use Python with OpenCV for video decoding (offline, no network)
2. Create a VideoLoader class that handles:
   - Opening video files
   - Validating format support
   - Extracting frames at configurable intervals
   - Proper error handling for unsupported formats
3. Support common formats: mp4, avi, mov, mkv, webm

## Acceptance Criteria
- [x] Accept local video file path
- [x] Decode video without network access
- [x] Surface clear errors for unsupported formats
