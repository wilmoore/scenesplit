# Contributing to SceneSplit

Thank you for your interest in contributing to SceneSplit! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, inclusive, and constructive in all interactions.

## Getting Started

### Prerequisites

- Rust (stable toolchain)
- OpenCV 4.x
- ONNX Runtime
- pkg-config

### macOS Setup

```bash
brew install opencv onnxruntime pkg-config llvm
export LIBCLANG_PATH=$(brew --prefix llvm)/lib
```

### Linux Setup (Ubuntu/Debian)

```bash
sudo apt-get install libopencv-dev clang libclang-dev pkg-config

# Install ONNX Runtime
ONNX_VERSION="1.16.3"
curl -L "https://github.com/microsoft/onnxruntime/releases/download/v${ONNX_VERSION}/onnxruntime-linux-x64-${ONNX_VERSION}.tgz" | tar xz
sudo mv onnxruntime-linux-x64-${ONNX_VERSION} /opt/onnxruntime
export ORT_LIB_LOCATION=/opt/onnxruntime/lib
```

### Building

```bash
git clone https://github.com/wilmoore/scenesplit.git
cd scenesplit
cargo build --release
```

### Running Tests

```bash
cargo test
```

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Use the bug report template
3. Include:
   - SceneSplit version (`scenesplit --version`)
   - Operating system and version
   - Steps to reproduce
   - Expected vs actual behavior
   - Sample video file info (format, duration, resolution) if applicable

### Suggesting Features

1. Check existing issues and discussions
2. Use the feature request template
3. Describe the use case and proposed solution

### Pull Requests

1. Fork the repository
2. Create a feature branch from `main`:
   ```bash
   git checkout -b feat/your-feature-name
   ```
3. Make your changes following the code style guidelines
4. Add tests for new functionality
5. Ensure all checks pass:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   cargo build --release
   ```
6. Commit with conventional commit messages:
   - `feat:` new features
   - `fix:` bug fixes
   - `docs:` documentation changes
   - `style:` formatting, no code change
   - `refactor:` code restructuring
   - `test:` adding tests
   - `chore:` maintenance tasks
7. Push and open a pull request

### Code Style

- Follow Rust idioms and best practices
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Write descriptive commit messages
- Add documentation for public APIs
- Keep functions focused and small

### Testing

- Add unit tests for new functionality
- Integration tests for CLI behavior
- Use `tempfile` for tests that create output files
- Ensure tests are deterministic

## Project Structure

```
scenesplit/
├── src/
│   ├── main.rs          # CLI entry point and argument parsing
│   ├── lib.rs           # Library exports
│   ├── video.rs         # Video decoding (OpenCV)
│   ├── embeddings.rs    # Frame embedding (ONNX/MobileNetV3)
│   ├── segmentation.rs  # Scene segmentation logic
│   ├── selection.rs     # Frame selection
│   ├── output.rs        # Image and metadata output
│   ├── model.rs         # ONNX model management
│   └── error.rs         # Error types
├── tests/               # Integration tests
├── doc/                 # Documentation and ADRs
└── .github/workflows/   # CI/CD configuration
```

## Questions?

Open a discussion or issue on GitHub.
