# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) documenting significant technical decisions.

## What is an ADR?

An ADR captures the context, decision, and consequences of an architecturally significant choice.

## Format

We use the [Michael Nygard format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions).

## Naming Convention

- Filename: `NNN-kebab-case-title.md` (e.g., `001-use-rust-for-implementation.md`)
- NNN = zero-padded sequence number (001, 002, 003...)
- Title in heading must match: `# NNN. Title`

## Index

- [001. Use Rust for Implementation](001-use-rust-for-implementation.md)
- [002. Use ONNX Runtime for ML Inference](002-use-onnx-runtime-for-ml-inference.md)
- [003. Auto-Download Model on First Run](003-auto-download-model-on-first-run.md)
