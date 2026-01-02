# Product Requirements Document (PRD)

**Product:** SceneSplit
**Type:** Offline CLI developer utility
**Version:** v1.0 MVP
**Author:** Spec Forge
**Mode:** Autopilot

---

## 1. Product Thesis

SceneSplit is an offline command-line tool that extracts a small, ordered set of semantically distinct still images from a video. The output represents meaningful state changes rather than raw frames or shot cuts. The product prioritizes semantic understanding, determinism, and minimal configuration, enabling developers and researchers to generate concise visual summaries without cloud services, GUIs, or manual filtering.

---

## 2. Core Design Principles

1. **Semantic over pixel**
   Frame selection is driven by embedding-level semantic change, not pixel deltas or cut detection alone.

2. **Minimal surface area**
   The CLI exposes exactly two primary user knobs, detail and quality. No advanced flags in MVP.

3. **Offline-first**
   All processing occurs locally. No network calls. No telemetry.

4. **Deterministic output**
   Identical inputs and configuration produce identical outputs.

5. **Narrative density control**
   Output is intentionally small and ordered, optimized for human review and downstream automation.

---

## 3. Personas

### P-001 Developer Integrator

* Builds tooling for video indexing, documentation, or LLM ingestion
* Needs automation-friendly outputs
* Values determinism and scriptability

### P-002 Researcher / Analyst

* Works with large video corpora
* Needs concise visual representations for analysis
* Limited tolerance for manual filtering

### P-003 Knowledge Worker

* Extracts insights or documentation from recorded content
* Needs fewer, more meaningful images
* Does not want to learn video editing tools

---

## 4. Input Scenarios

* Large archive of recorded meetings or lectures
* Screen recordings showing stateful workflows
* Field or lab recordings with slow visual evolution
* Videos destined for LLM or multimodal pipelines
* CI or batch workflows that require visual summaries

---

## 5. User Journeys

### J-001 Extract Semantic Keyframes

User provides a video file and receives a minimal, ordered set of representative still images.

### J-002 Control Granularity

User adjusts detail level to receive fewer or more semantic states without tuning thresholds.

### J-003 Optimize for Performance or Quality

User selects quality preset based on available compute and fidelity needs.

---

## 6. UX Surface Inventory

| Screen ID | Surface        | Description                                        |
| --------- | -------------- | -------------------------------------------------- |
| S-001     | CLI Invocation | Entry point via terminal                           |
| S-002     | Console Output | Logs, progress, and summary                        |
| S-003     | File Output    | Directory containing extracted stills and metadata |

---

## 7. Behavior and Editing Model

* SceneSplit processes video sequentially.
* Candidate frames are embedded using a pretrained vision model.
* Semantic change is detected via similarity deltas and clustering.
* One representative frame is selected per semantic segment.
* Frames are ordered by timestamp.
* No interactive editing in MVP.
* No post-hoc deletion or merging in MVP.

---

## 8. Constraints and Anti-Features

### Constraints

* Offline execution only
* CLI-only interface
* Cross-platform support limited to Linux and macOS in MVP
* Lean dependency footprint where feasible

### Explicit Anti-Features (Out of Scope)

* OCR
* Caption generation
* Slide decks
* Cloud APIs
* GUI
* Audio or transcript processing
* Manual frame selection

---

## 9. Success and Failure Criteria

### Success

* Users receive 5 to 30 images for long-form videos by default
* Images correspond to meaningful semantic changes
* Output usable without manual filtering
* Deterministic repeat runs

### Failure

* Overgeneration similar to frame dumps
* Under-generation that misses major state changes
* Non-deterministic outputs
* Hidden cloud dependency

---

## 10. North Star Metric

**Median number of extracted stills per 30-minute video while maintaining user-reported semantic coverage ≥ 0.8**

Secondary indicators:

* Zero required manual deletions
* Adoption in scripted pipelines

---

## 11. Epics

* E-001 [MUST] Video ingestion and decoding
* E-002 [MUST] Semantic embedding and change detection
* E-003 [MUST] Deterministic frame selection
* E-004 [MUST] CLI interface and configuration
* E-005 [SHOULD] Metadata output
* E-006 [COULD] Packaging and distribution
* E-007 [WONT] GUI or interactive review

---

## 12. User Stories with Acceptance Criteria

### E-001 Video Ingestion

* US-001 [MUST] As a user, I can provide a local video file as input
  **Given** a valid video path
  **When** I run scenesplit
  **Then** the video is decoded without network access
  **And** errors are surfaced for unsupported formats

---

### E-002 Semantic Processing

* US-002 [MUST] As a user, I get frames grouped by semantic similarity
  **Given** decoded frames
  **When** embeddings are computed
  **Then** frames are clustered by semantic change
  **And** pixel-only noise does not create new segments

---

### E-003 Frame Selection

* US-003 [MUST] As a user, I receive one representative still per semantic segment
  **Given** a semantic segment
  **When** selection occurs
  **Then** exactly one frame is output per segment
  **And** ordering matches video timeline

---

### E-004 CLI Interface

* US-004 [MUST] As a user, I can control granularity with --detail
  **Given** `--detail key|summary|all`
  **When** the command runs
  **Then** output size scales predictably with detail level

* US-005 [MUST] As a user, I can control performance with --quality
  **Given** `--quality fast|balanced|best`
  **When** the command runs
  **Then** processing time and fidelity reflect the preset

---

### E-005 Metadata

* US-006 [SHOULD] As a user, I receive minimal metadata per image
  **Given** extracted stills
  **Then** timestamp and segment index are included
  **And** metadata is machine-readable

---

## 13. Traceability Map

| Story  | Epic  | Journey | Screen | Priority |
| ------ | ----- | ------- | ------ | -------- |
| US-001 | E-001 | J-001   | S-001  | MUST     |
| US-002 | E-002 | J-001   | S-002  | MUST     |
| US-003 | E-003 | J-001   | S-003  | MUST     |
| US-004 | E-004 | J-002   | S-001  | MUST     |
| US-005 | E-004 | J-003   | S-001  | MUST     |
| US-006 | E-005 | J-001   | S-003  | SHOULD   |

---

## 14. Lo-fi UI Mockups (ASCII)

### S-001 CLI Invocation

```
$ scenesplit input.mp4 --detail summary --quality balanced

Processing video...
Embedding frames...
Detecting semantic changes...
Extracted 12 stills
Output written to ./scenesplit_output/
```

States:

* Empty: no args → usage message
* Loading: progress logs
* Error: invalid file, unsupported codec
* Success: summary and output path

---

### S-003 Output Directory

```
scenesplit_output/
├── 0001.jpg
├── 0002.jpg
├── 0003.jpg
└── metadata.json
```

---

## 15. Decision Log

### D-001 Embedding-based semantic detection

* **Options:** Pixel diff, shot cuts, vision embeddings
* **Winner:** Vision embeddings
* **Evidence:** Idea Pack sections 6, 7, 9
* **Confidence:** 0.90

### D-002 CLI-only interface

* **Options:** CLI, GUI, hybrid
* **Winner:** CLI
* **Evidence:** Target users, principles
* **Confidence:** 0.85

### D-003 Two-knob configuration

* **Options:** Many flags, config file, two knobs
* **Winner:** Two knobs
* **Evidence:** Core solution, principles
* **Confidence:** 0.80

### D-004 Offline-only MVP

* **Options:** Offline, cloud-assisted
* **Winner:** Offline
* **Evidence:** Principles, risks
* **Confidence:** 0.88

---

## 16. Assumptions

* MVP timebox: 2 to 4 weeks
* Budget posture: lean
* Platform: web not required, CLI only
* Supported OS: Linux and macOS initially
* Users are comfortable installing CLI tools
* Pretrained embedding models are acceptable dependencies

---

> **This PRD is complete.**
> Copy this Markdown into Word, Google Docs, Notion, or directly into a coding model.
