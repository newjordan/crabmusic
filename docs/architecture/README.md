# CrabMusic Architecture Documentation

This directory contains **sharded architecture sections** extracted from the master architecture document for quick reference during development.

---

## Purpose

The full architecture document (`docs/architecture.md`) is comprehensive but large. These sharded files provide:

1. **Quick Reference**: Load only relevant sections during development
2. **Dev Agent Optimization**: Smaller context windows for AI agents
3. **Focused Documentation**: Each file covers one architectural concern

---

## File Index

### 📘 Always-Loaded Files (Dev Agent)
These files are automatically loaded by dev agents per `core-config.yaml`:

- **[tech-stack.md](tech-stack.md)** - Technology stack table with versions and rationale
- **[coding-standards.md](coding-standards.md)** - Mandatory coding rules and conventions
- **[source-tree.md](source-tree.md)** - Complete directory structure and organization

### 📗 Reference Files
Load as needed based on task:

- **[components-overview.md](components-overview.md)** - Component descriptions and interfaces *(needs creation)*
- **[data-models.md](data-models.md)** - Data structures and relationships *(needs creation)*
- **[error-handling.md](error-handling.md)** - Error handling patterns and strategies *(needs creation)*

---

## When to Use Which File

### Starting a New Story
**Load**: `tech-stack.md`, `coding-standards.md`, `source-tree.md`

These give you everything needed to start implementing:
- What technologies to use
- How to write code
- Where to put files

### Implementing Audio Features
**Load**: `components-overview.md` (Audio Capture section)

Get details on:
- AudioCaptureDevice trait
- CPAL integration
- Ring buffer design

### Implementing DSP Features
**Load**: `components-overview.md` (DSP Processing section)

Get details on:
- DspProcessor interface
- FFT implementation
- AudioParameters structure

### Implementing Visualizations
**Load**: `components-overview.md` (Visualization Engine section)

Get details on:
- Visualizer trait
- GridBuffer structure
- Coverage algorithms

### Handling Errors
**Load**: `error-handling.md`

Get patterns for:
- Error type definitions
- Error propagation
- User-facing error messages

---

## Relationship to Master Document

**Master Document**: `docs/architecture.md`
- **Purpose**: Complete, authoritative architecture reference
- **Audience**: Architects, technical leads, comprehensive review
- **When to use**: Architecture changes, complete system understanding

**Sharded Files**: `docs/architecture/*.md`
- **Purpose**: Quick reference during implementation
- **Audience**: Developers, AI dev agents
- **When to use**: Daily development, implementing stories

**Rule**: Sharded files are **extracted from** master document
- Keep shards in sync with master
- Master document is source of truth
- Shards are convenience copies

---

## Keeping Documents in Sync

When updating architecture:

1. **Update master first**: Edit `docs/architecture.md`
2. **Extract to shards**: Update relevant shard files
3. **Verify consistency**: Ensure no contradictions
4. **Update this index**: If adding/removing shards

**Note**: Consider using an agent to automate shard extraction from master document

---

## Creating New Shards

To add a new architecture shard:

1. Identify a frequently-referenced section from master document
2. Extract section to focused file (e.g., `database-schema.md`)
3. Add to this README index
4. Optionally update `core-config.yaml` if always-loaded

**Guidelines for good shards**:
- **Self-contained**: Can be understood independently
- **Focused**: Single architectural concern
- **Actionable**: Provides specific implementation guidance
- **Right-sized**: 200-500 lines ideal (not too small, not too large)

---

## Core Config Integration

The `core-config.yaml` specifies which files are always loaded by dev agents:

```yaml
devLoadAlwaysFiles:
  - docs/architecture/coding-standards.md
  - docs/architecture/tech-stack.md
  - docs/architecture/source-tree.md
```

**To add a shard to always-load**:
1. Create the shard file
2. Add to `devLoadAlwaysFiles` in `.bmad-core/core-config.yaml`
3. Test with dev agent to ensure it loads correctly

**Keep always-load list minimal**:
- Only include files needed for >80% of development tasks
- Large always-load lists consume context unnecessarily
- Most shards should be loaded on-demand

---

## Quick Links

- **Master Architecture**: [../architecture.md](../architecture.md)
- **Implementation Plan**: [../implementation-plan.md](../implementation-plan.md)
- **Stories Directory**: [../stories/](../stories/)
- **Brainstorming Session**: [../brainstorming-session-results.md](../brainstorming-session-results.md)

---

**For Developers**: Start with the always-loaded files, then reference others as needed during implementation.

**For Architects**: Edit the master document first, then update shards to keep them in sync.
