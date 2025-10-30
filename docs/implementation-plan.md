# CrabMusic Implementation Plan

**Status**: Draft - Ready for Agent Completion
**Created**: 2025-10-29
**Architecture Reference**: [docs/architecture.md](architecture.md)

---

## Overview

This document provides a structured implementation plan for **crabmusic**, breaking down the architecture into deliverable epics and stories. The plan follows a progressive complexity approach: build foundation → prove concept → expand capabilities.

### MVP Scope

**Goal**: Validate the entire system architecture with minimal but satisfying implementation

**MVP Definition**: Sine wave visualization that smoothly reacts to system audio playback

**MVP Success Criteria**:
- ✅ System audio captured without user configuration
- ✅ Audio-reactive sine wave renders at 60 FPS
- ✅ Visual response feels natural and satisfying
- ✅ Configuration can be adjusted without recompilation
- ✅ Runs on at least Linux and macOS

**Out of Scope for MVP**:
- Multiple visualization modes
- Color support
- Interactive controls
- Advanced effects (noise, turbulence)
- Sacred geometry patterns

---

## Epic Breakdown

### Epic 1: Project Foundation
**Goal**: Set up project structure, dependencies, and development workflow
**Priority**: P0 (Blocking)
**Estimated Effort**: 1-2 days

**Stories**:
1. [FOUND-001](stories/FOUND-001-project-setup.md) - Project Setup and Scaffolding
2. [FOUND-002](stories/FOUND-002-ci-pipeline.md) - CI/CD Pipeline Setup

---

### Epic 2: Audio Capture System
**Goal**: Capture system audio output in real-time with cross-platform support
**Priority**: P0 (Blocking)
**Estimated Effort**: 3-5 days

**Stories**:
1. [AUDIO-001](stories/AUDIO-001-audio-capture-trait.md) - Define Audio Capture Interface
2. [AUDIO-002](stories/AUDIO-002-cpal-implementation.md) - Implement CPAL Audio Capture
3. [AUDIO-003](stories/AUDIO-003-ring-buffer.md) - Ring Buffer for Audio Pipeline
4. [AUDIO-004](stories/AUDIO-004-audio-testing.md) - Audio Capture Testing & Validation

**Dependencies**: FOUND-001 must be complete

---

### Epic 3: DSP Processing
**Goal**: Extract meaningful audio parameters from captured audio
**Priority**: P0 (Blocking)
**Estimated Effort**: 4-6 days

**Stories**:
1. [DSP-001](stories/DSP-001-fft-processor.md) - FFT Processor Implementation
2. [DSP-002](stories/DSP-002-frequency-bands.md) - Frequency Band Extraction
3. [DSP-003](stories/DSP-003-parameter-smoothing.md) - Parameter Smoothing & Windowing
4. [DSP-004](stories/DSP-004-beat-detection.md) - Beat Detection (Post-MVP)
5. [DSP-005](stories/DSP-005-dsp-testing.md) - DSP Testing with Synthetic Audio

**Dependencies**: AUDIO-003 must be complete (need audio buffers)

---

### Epic 4: Visualization Engine
**Goal**: Transform audio parameters into visual grid representations
**Priority**: P0 (Blocking for MVP)
**Estimated Effort**: 5-7 days

**Stories**:
1. [VIZ-001](stories/VIZ-001-grid-buffer.md) - Grid Buffer Data Structure
2. [VIZ-002](stories/VIZ-002-character-sets.md) - Character Set Definitions
3. [VIZ-003](stories/VIZ-003-coverage-algorithm.md) - Character Coverage Algorithm
4. [VIZ-004](stories/VIZ-004-visualizer-trait.md) - Visualizer Trait Design
5. [VIZ-005](stories/VIZ-005-sine-wave-visualizer.md) - Sine Wave Visualizer (MVP)
6. [VIZ-006](stories/VIZ-006-spectrum-visualizer.md) - Spectrum Analyzer Visualizer (Post-MVP)
7. [VIZ-007](stories/VIZ-007-oscilloscope-visualizer.md) - Oscilloscope Visualizer (Post-MVP)

**Dependencies**: DSP-002 must be complete (need audio parameters)

---

### Epic 5: Terminal Rendering
**Goal**: Efficiently render visualizations to terminal display
**Priority**: P0 (Blocking)
**Estimated Effort**: 3-4 days

**Stories**:
1. [RENDER-001](stories/RENDER-001-terminal-init.md) - Terminal Initialization & Cleanup
2. [RENDER-002](stories/RENDER-002-ratatui-integration.md) - Ratatui Integration
3. [RENDER-003](stories/RENDER-003-differential-rendering.md) - Differential Rendering Optimization
4. [RENDER-004](stories/RENDER-004-terminal-resize.md) - Terminal Resize Handling

**Dependencies**: VIZ-001 must be complete (need grid buffer)

---

### Epic 6: Configuration System
**Goal**: User-configurable settings with hot-reload capability
**Priority**: P1 (Required for MVP)
**Estimated Effort**: 2-3 days

**Stories**:
1. [CONFIG-001](stories/CONFIG-001-config-structure.md) - Configuration Data Structure
2. [CONFIG-002](stories/CONFIG-002-yaml-loading.md) - YAML Config Loading & Validation
3. [CONFIG-003](stories/CONFIG-003-hot-reload.md) - Hot-Reload File Watching
4. [CONFIG-004](stories/CONFIG-004-default-config.md) - Default Configuration & Examples

**Dependencies**: Can develop in parallel with other epics

---

### Epic 7: Pipeline Integration
**Goal**: Connect all components into functioning application
**Priority**: P0 (Blocking)
**Estimated Effort**: 3-5 days

**Stories**:
1. [PIPELINE-001](stories/PIPELINE-001-main-loop.md) - Main Application Loop
2. [PIPELINE-002](stories/PIPELINE-002-thread-coordination.md) - Thread Coordination & Synchronization
3. [PIPELINE-003](stories/PIPELINE-003-error-handling.md) - Error Handling & Recovery
4. [PIPELINE-004](stories/PIPELINE-004-cli-interface.md) - CLI Argument Parsing
5. [PIPELINE-005](stories/PIPELINE-005-logging-setup.md) - Logging & Diagnostics Setup

**Dependencies**: All component epics must be substantially complete

---

### Epic 8: Testing & Validation
**Goal**: Comprehensive testing and performance validation
**Priority**: P1 (Required before release)
**Estimated Effort**: 4-6 days

**Stories**:
1. [TEST-001](stories/TEST-001-unit-test-suite.md) - Unit Test Suite Completion
2. [TEST-002](stories/TEST-002-integration-tests.md) - Integration Tests
3. [TEST-003](stories/TEST-003-performance-benchmarks.md) - Performance Benchmarks
4. [TEST-004](stories/TEST-004-cross-platform-testing.md) - Cross-Platform Testing
5. [TEST-005](stories/TEST-005-audio-validation.md) - Audio Validation with Real Music

**Dependencies**: PIPELINE-003 must be complete

---

### Epic 9: Documentation & Release
**Goal**: User-facing documentation and release preparation
**Priority**: P1 (Required for MVP release)
**Estimated Effort**: 2-3 days

**Stories**:
1. [DOCS-001](stories/DOCS-001-user-readme.md) - User-Facing README
2. [DOCS-002](stories/DOCS-002-configuration-guide.md) - Configuration Guide
3. [DOCS-003](stories/DOCS-003-troubleshooting.md) - Troubleshooting Guide
4. [DOCS-004](stories/DOCS-004-contributing.md) - Contributing Guidelines
5. [RELEASE-001](stories/RELEASE-001-build-release.md) - Build & Release Process

**Dependencies**: TEST-005 must be complete

---

## Critical Path

The **critical path** for MVP (longest dependency chain):

```
FOUND-001 → AUDIO-001 → AUDIO-002 → AUDIO-003 → DSP-001 → DSP-002 →
VIZ-001 → VIZ-003 → VIZ-004 → VIZ-005 → RENDER-001 → RENDER-002 →
PIPELINE-001 → PIPELINE-002 → TEST-005
```

**Estimated Critical Path Duration**: 18-25 days of focused development

---

## Parallel Work Streams

These stories can be developed **in parallel** to reduce overall time:

**Stream 1 - Core Pipeline** (Critical Path):
- Audio → DSP → Visualization → Rendering

**Stream 2 - Configuration** (Parallel):
- CONFIG-001 → CONFIG-002 → CONFIG-003
- Can integrate once pipeline is working

**Stream 3 - Infrastructure** (Parallel):
- FOUND-002 (CI/CD)
- TEST-001, TEST-002 (test infrastructure)

**Stream 4 - Documentation** (Parallel):
- DOCS-001, DOCS-002 can be drafted early
- Final edits after implementation complete

---

## Sprint/Milestone Suggestions

### Milestone 1: Foundation Complete
**Duration**: Sprint 1 (Week 1)
**Stories**: FOUND-001, FOUND-002, CONFIG-001
**Goal**: Project structure ready, CI working, basic config loading
**Demo**: Show project builds, tests run, config loads

---

### Milestone 2: Audio & DSP Working
**Duration**: Sprint 2-3 (Week 2-3)
**Stories**: AUDIO-001 through AUDIO-004, DSP-001 through DSP-003
**Goal**: Audio captured, FFT working, parameters extracted
**Demo**: Log audio parameters in real-time (no visualization yet)

---

### Milestone 3: Static Visualization
**Duration**: Sprint 3-4 (Week 3-4)
**Stories**: VIZ-001 through VIZ-005, RENDER-001, RENDER-002
**Goal**: Sine wave renders in terminal (static, no audio yet)
**Demo**: Show sine wave with hardcoded parameters

---

### Milestone 4: MVP Integration
**Duration**: Sprint 4-5 (Week 4-5)
**Stories**: PIPELINE-001 through PIPELINE-005, CONFIG-002, CONFIG-003
**Goal**: Full pipeline working - audio drives visualization
**Demo**: 🎉 **MVP COMPLETE** - Sine wave reacts to music!

---

### Milestone 5: Polish & Release
**Duration**: Sprint 6 (Week 6)
**Stories**: TEST-001 through TEST-005, DOCS-001 through DOCS-004, RELEASE-001
**Goal**: Tested, documented, releasable
**Demo**: Public MVP release

---

## Post-MVP Roadmap

After MVP validation, prioritize based on user feedback:

**Phase 2 - Visualization Expansion**:
- VIZ-006 (Spectrum Analyzer)
- VIZ-007 (Oscilloscope modes)
- DSP-004 (Beat Detection for reactive visuals)

**Phase 3 - Advanced Features**:
- Color support for modern terminals
- Interactive controls (keyboard shortcuts)
- Preset system for different music genres
- Recording/export to file

**Phase 4 - Sacred Geometry**:
- Flower of Life renderer
- Mandala/Kaleidoscope patterns
- Recursive nested patterns
- Integration with existing sacred geometry work

---

## Story File Format

Each story file follows this template:

```markdown
# [STORY-ID] Story Title

**Epic**: Epic Name
**Priority**: P0/P1/P2
**Estimated Effort**: X days
**Status**: Not Started | In Progress | Complete

## Description

[What needs to be built and why]

## Acceptance Criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

## Technical Approach

[High-level approach, key decisions, architecture references]

## Dependencies

- Depends on: [Other story IDs]
- Blocks: [Stories that depend on this]

## Architecture References

- [Relevant architecture sections]

## Testing Requirements

- Unit tests: [What to test]
- Integration tests: [What to test]

## Notes for AI Agent

[Specific guidance for implementing this story]
```

---

## Next Steps

### For Human Review:
1. ✅ Review epic breakdown - does it match your vision?
2. ✅ Validate MVP scope - is sine wave enough to prove concept?
3. ✅ Check critical path - does timeline feel reasonable?
4. ✅ Approve story structure before agent completion

### For Agent Completion:
1. Each story file needs detailed **Description** section
2. Specific **Acceptance Criteria** (currently placeholders)
3. **Technical Approach** with architecture references
4. **Testing Requirements** details
5. **Notes for AI Agent** with implementation guidance

### After Story Completion:
1. Use dev agent to implement stories following plan
2. Track progress in story status fields
3. Adjust estimates and dependencies as you learn
4. Celebrate MVP completion! 🎉

---

## Questions to Resolve

- **Q1**: Should beat detection (DSP-004) be in MVP or post-MVP?
  - *Current decision*: Post-MVP (not needed for sine wave validation)

- **Q2**: Hot-reload (CONFIG-003) - MVP or post-MVP?
  - *Current decision*: MVP (greatly improves development experience)

- **Q3**: Windows support in MVP or post-MVP?
  - *Current decision*: Post-MVP (focus on Linux/macOS first, CPAL abstracts Windows anyway)

---

*This implementation plan is a living document. Update as you learn during development.*
