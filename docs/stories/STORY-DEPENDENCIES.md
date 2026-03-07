# CrabMusic Story Dependencies & Work Order
## Sequential Implementation Guide

**Purpose**: Visual dependency tree showing implementation order
**Created**: 2025-10-31

---

## Legend

- ✅ = Already implemented
- 🔵 = Foundation (P1 - High Priority)
- 🟢 = Ready to implement (dependencies met)
- 🟡 = Blocked (waiting on dependencies)
- 🔴 = Future/Lower Priority

---

## Current Status: Foundation Complete

```
✅ FOUND-001: Project Setup
✅ AUDIO-001: Audio Capture Trait
✅ AUDIO-002: CPAL Implementation
✅ AUDIO-003: Ring Buffer
✅ DSP-001: FFT Processor
✅ DSP-002: Frequency Bands
✅ DSP-004: Beat Detection (Energy-based)
✅ VIZ-001: Grid Buffer
✅ VIZ-003: Coverage Algorithm
✅ VIZ-004: Visualizer Trait
✅ VIZ-005: Sine Wave Visualizer
✅ VIZ-006: Spectrum Analyzer
✅ VIZ-007: Oscilloscope
✅ VIZ-008: Braille Video Playback
✅ RENDER-001: Terminal Init
✅ RENDER-002: Ratatui Integration
✅ PIPELINE-001: Main Loop
✅ CONFIG-001: Config Structure
✅ CONFIG-002: YAML Loading
✅ Color System: 6 color schemes
✅ Mode Switching: Keys 1-5
```

---

## Phase 3: Effects & Enhanced Visuals

### Track 1: Effects Pipeline (Foundation)

```
🟢 EFFECTS-007: Effect Pipeline Framework
    ├─ Dependencies: VIZ-001 ✅, VIZ-004 ✅
    ├─ Blocks: All other EFFECTS stories
    └─ Priority: P1 (Start here!)

    ├─🟡 EFFECTS-001: Perlin Noise Layer
    │   ├─ Dependencies: EFFECTS-007
    │   └─ Priority: P2
    │
    ├─🟡 EFFECTS-002: Turbulence Field
    │   ├─ Dependencies: EFFECTS-001
    │   └─ Priority: P2
    │
    ├─🟡 EFFECTS-003: Bloom Effect
    │   ├─ Dependencies: EFFECTS-007
    │   └─ Priority: P2
    │
    ├─🟡 EFFECTS-004: Scanline Effect
    │   ├─ Dependencies: EFFECTS-007
    │   └─ Priority: P2
    │
    ├─🟡 EFFECTS-005: CRT Curve & Distortion
    │   ├─ Dependencies: EFFECTS-007
    │   └─ Priority: P2
    │
    └─🟡 EFFECTS-006: Phosphor Glow
        ├─ Dependencies: EFFECTS-007
        └─ Priority: P2
```

**Implementation Order**:
1. EFFECTS-007 (Framework) ← **Start here**
2. EFFECTS-001, 003, 004, 006 (can be parallel)
3. EFFECTS-002 (needs EFFECTS-001)
4. EFFECTS-005 (can be anytime after framework)

---

### Track 2: Color System Enhancement (Parallel with Track 1)

```
🟢 COLOR-001: Advanced Color Schemes
    ├─ Dependencies: Current color system ✅
    ├─ Blocks: COLOR-002, COLOR-003
    └─ Priority: P2

    ├─🟡 COLOR-002: Dynamic Gradient Generator
    │   ├─ Dependencies: COLOR-001
    │   ├─ Blocks: COLOR-003
    │   └─ Priority: P2
    │
    └─🟡 COLOR-003: Frequency-to-Color Mapping
        ├─ Dependencies: COLOR-002, DSP-002 ✅
        └─ Priority: P2
```

**Implementation Order**:
1. COLOR-001 (can start immediately)
2. COLOR-002
3. COLOR-003

---

### Track 3: Additional Visualizers (Parallel with Tracks 1 & 2)

```
🟢 VIZ-009: Character Coverage Test Suite
    ├─ Dependencies: VIZ-001 ✅, VIZ-003 ✅
    └─ Priority: P1 (Good starting point)

🟢 VIZ-010: Matrix Rain Effect
    ├─ Dependencies: VIZ-004 ✅
    └─ Priority: P2

🟢 VIZ-011: Cellular Automata Visualizer
    ├─ Dependencies: VIZ-004 ✅
    └─ Priority: P2

🟢 VIZ-021: Pulsing Blob Visualizer
    ├─ Dependencies: VIZ-003 ✅, VIZ-004 ✅
    └─ Priority: P2

🟢 VIZ-018: Enhanced Lissajous Curves
    ├─ Dependencies: VIZ-007 ✅
    └─ Priority: P2

🟢 VIZ-019: Spiral Patterns
    ├─ Dependencies: VIZ-003 ✅, VIZ-004 ✅
    └─ Priority: P2

🟢 VIZ-020: Circular Waves
    ├─ Dependencies: VIZ-003 ✅, VIZ-004 ✅
    └─ Priority: P2
```

**Implementation Order**: Any order (all dependencies met)

---

## Phase 4: Sacred Geometry (HIGH PRIORITY - User Interest)

```
🟢 VIZ-013: Flower of Life Visualizer
    ├─ Dependencies: VIZ-003 ✅, VIZ-004 ✅
    ├─ Blocks: VIZ-016 (partially)
    └─ Priority: P1 ⭐ HIGH USER INTEREST

🟢 VIZ-014: Mandala Generator
    ├─ Dependencies: VIZ-003 ✅, VIZ-004 ✅
    ├─ Blocks: VIZ-016 (partially)
    └─ Priority: P1 ⭐ HIGH USER INTEREST

🟢 VIZ-015: Kaleidoscope Patterns
    ├─ Dependencies: VIZ-003 ✅, VIZ-004 ✅
    ├─ Blocks: VIZ-016 (partially)
    └─ Priority: P1 ⭐ HIGH USER INTEREST

    └─🟡 VIZ-016: Recursive Pattern System
        ├─ Dependencies: VIZ-013, VIZ-014, VIZ-015
        └─ Priority: P2
```

**Implementation Order**:
1. VIZ-013 (Flower of Life) ← **User excitement area!**
2. VIZ-014 (Mandala)
3. VIZ-015 (Kaleidoscope)
4. VIZ-016 (Recursive System) ← Needs all three above

**Note**: This is a prime candidate for **starting implementation** given user interest in sacred geometry!

---

## Phase 5: Interaction & Composition

```
🟢 VIZ-012: Element Interaction Framework
    ├─ Dependencies: VIZ-004 ✅, EFFECTS-007 🟡
    ├─ Blocks: VIZ-017, EFFECTS-008, EFFECTS-009
    └─ Priority: P1

    ├─🟡 EFFECTS-008: Effect Parameter Modulation
    │   ├─ Dependencies: EFFECTS-007, DSP-002 ✅
    │   └─ Priority: P2
    │
    ├─🟡 EFFECTS-009: Effect Compositing System
    │   ├─ Dependencies: EFFECTS-007, EFFECTS-008
    │   └─ Priority: P2
    │
    └─🟡 VIZ-017: Generative Composition Engine
        ├─ Dependencies: VIZ-012, All visualizers
        └─ Priority: P2
```

**Implementation Order**:
1. Wait for EFFECTS-007 to complete
2. VIZ-012 (Interaction Framework)
3. EFFECTS-008, EFFECTS-009 (can be parallel)
4. VIZ-017 (Composition Engine) ← Ambitious moonshot

---

## Phase 6: Controls & Presets

```
🟢 CONTROL-001: Keyboard Control System
    ├─ Dependencies: Pipeline ✅
    ├─ Blocks: CONTROL-002
    └─ Priority: P1

    └─🟡 CONTROL-002: Real-Time Parameter Adjustment
        ├─ Dependencies: CONTROL-001, CONFIG-003 ✅
        └─ Priority: P1

🟢 CONFIG-005: Preset System Framework
    ├─ Dependencies: CONFIG-001 ✅, CONFIG-002 ✅
    ├─ Blocks: CONFIG-006, CONFIG-007, CONFIG-008, CONFIG-009
    └─ Priority: P1

    ├─🟡 CONFIG-006: Genre-Specific Presets
    │   ├─ Dependencies: CONFIG-005
    │   └─ Priority: P2
    │
    ├─🟡 CONFIG-007: User Preset Manager
    │   ├─ Dependencies: CONFIG-005
    │   └─ Priority: P2
    │
    ├─🟡 CONFIG-008: Preset Import/Export
    │   ├─ Dependencies: CONFIG-005, CONFIG-007
    │   └─ Priority: P2
    │
    └─🟡 CONFIG-009: Visual Preset Editor (TUI)
        ├─ Dependencies: CONFIG-005, CONTROL-002
        └─ Priority: P2
```

**Implementation Order**:
1. CONTROL-001 (can start immediately)
2. CONTROL-002
3. CONFIG-005 (can start immediately)
4. CONFIG-006, 007 (parallel)
5. CONFIG-008
6. CONFIG-009 (after CONTROL-002)

---

## Phase 7: Export & Recording

```
🟢 EXPORT-001: Frame Capture System
    ├─ Dependencies: VIZ-001 ✅
    ├─ Blocks: All other EXPORT stories
    └─ Priority: P1

    ├─🟡 EXPORT-002: ASCII Animation Recorder
    │   ├─ Dependencies: EXPORT-001
    │   └─ Priority: P1
    │
    ├─🟡 EXPORT-003: Video Export (FFmpeg)
    │   ├─ Dependencies: EXPORT-001
    │   └─ Priority: P2
    │
    ├─🟡 EXPORT-004: Screenshot Feature
    │   ├─ Dependencies: EXPORT-001
    │   └─ Priority: P2
    │
    ├─🟡 EXPORT-005: GIF Export
    │   ├─ Dependencies: EXPORT-001
    │   └─ Priority: P2
    │
    └─🟡 EXPORT-006: Playback Mode
        ├─ Dependencies: EXPORT-002
        └─ Priority: P2
```

**Implementation Order**:
1. EXPORT-001 (Foundation)
2. EXPORT-002, 003, 004, 005 (can be parallel)
3. EXPORT-006

---

## Phase 8: Advanced DSP

```
🟢 DSP-007: Harmonic Analysis
    ├─ Dependencies: DSP-001 ✅, DSP-002 ✅
    ├─ Blocks: DSP-008
    └─ Priority: P2

    └─🟡 DSP-008: Pitch Detection
        ├─ Dependencies: DSP-007
        └─ Priority: P2

🟢 DSP-009: Spectral Centroid
    ├─ Dependencies: DSP-001 ✅
    └─ Priority: P2

🟢 DSP-013: Transient Detection
    ├─ Dependencies: DSP-001 ✅
    └─ Priority: P2

🟡 DSP-010: Audio Feature Extraction Framework
    ├─ Dependencies: DSP-007, DSP-008, DSP-009, DSP-013
    └─ Priority: P2
```

**Implementation Order**:
1. DSP-007, DSP-009, DSP-013 (can be parallel)
2. DSP-008 (needs DSP-007)
3. DSP-010 (Framework - needs all above)

---

## Phase 9: Performance & Optimization

```
🟢 PERF-001: Rendering Performance Profiling
    ├─ Dependencies: Pipeline ✅
    └─ Priority: P1

🟢 PERF-003: Multi-threaded Visual Processing
    ├─ Dependencies: VIZ-004 ✅
    └─ Priority: P2

🟢 PERF-004: Memory Pool Optimization
    ├─ Dependencies: VIZ-001 ✅
    └─ Priority: P2
```

**Implementation Order**: Any order (all dependencies met)

---

## Phase 10: Testing & Documentation

```
🟢 TEST-006: Visual Regression Testing
    ├─ Dependencies: All visualizers
    └─ Priority: P1

🟢 TEST-009: Character Coverage Validation
    ├─ Dependencies: VIZ-003 ✅
    └─ Priority: P2

🟡 DOCS-005: Visual Mode Gallery
    ├─ Dependencies: All visualizers (ongoing)
    └─ Priority: P2

🟡 DOCS-007: Preset Creation Guide
    ├─ Dependencies: CONFIG-005
    └─ Priority: P2

🟡 DOCS-010: Sacred Geometry Math Reference
    ├─ Dependencies: VIZ-013, VIZ-014, VIZ-015
    └─ Priority: P2
```

**Implementation Order**: As features are completed

---

## Recommended Parallel Work Streams

### Stream A: Effects Track (3-4 weeks)
```
Week 1: EFFECTS-007 (Framework)
Week 2: EFFECTS-001, 003, 004 (parallel)
Week 3: EFFECTS-002, 005, 006
Week 4: Polish & integration
```

### Stream B: Sacred Geometry Track (4-6 weeks) ⭐ **RECOMMENDED START**
```
Week 1-2: VIZ-013 (Flower of Life)
Week 3-4: VIZ-014 (Mandala)
Week 5: VIZ-015 (Kaleidoscope)
Week 6: VIZ-016 (Recursive System)
```

### Stream C: Color Enhancement Track (2-3 weeks)
```
Week 1: COLOR-001 (Advanced schemes)
Week 2: COLOR-002 (Gradients)
Week 3: COLOR-003 (Frequency mapping)
```

### Stream D: Additional Visualizers (2-4 weeks)
```
Week 1: VIZ-009 (Test suite)
Week 2: VIZ-010, VIZ-011 (parallel)
Week 3: VIZ-018, VIZ-019, VIZ-020 (parallel)
Week 4: VIZ-021
```

---

## Quick Start: Top 3 Recommended Starting Points

### Option 1: Sacred Geometry First (Highest User Interest)
```
1. Create stories: VIZ-013, VIZ-014, VIZ-015, VIZ-016
2. Implement: VIZ-013 (Flower of Life)
3. Demo to user → Get feedback
4. Continue with VIZ-014, VIZ-015
```
**Why**: Direct alignment with user excitement, all dependencies met

### Option 2: Effects Pipeline First (Foundation Building)
```
1. Create stories: EFFECTS-007, 001, 003, 004, 006
2. Implement: EFFECTS-007 (Framework)
3. Implement: EFFECTS-003 (Bloom) for visual wow factor
4. Continue with other effects
```
**Why**: Unlocks multiplicative value (effects × all visualizers)

### Option 3: Color Enhancement First (Quick Wins)
```
1. Create stories: COLOR-001, 002, 003
2. Implement all three sequentially
3. Add retro CGA/EGA palettes
4. Frequency-to-color mapping
```
**Why**: Fast completion, high visual impact, low complexity

---

## Complete Sequential Implementation Order (If doing everything)

**Phase 3 (Parallel tracks possible)**:
1. EFFECTS-007 + COLOR-001 + VIZ-009 (parallel)
2. Sacred Geometry: VIZ-013 → VIZ-014 → VIZ-015
3. Effects expansion: EFFECTS-001/003/004/006 (parallel)
4. COLOR-002 → COLOR-003
5. Additional visualizers: VIZ-010, 011, 018-021 (parallel)
6. VIZ-016 (Recursive patterns)

**Phase 4**:
7. EFFECTS-002, 005 (remaining effects)
8. VIZ-012 (Interaction framework)
9. EFFECTS-008 → EFFECTS-009

**Phase 5**:
10. CONTROL-001 → CONTROL-002
11. CONFIG-005 → CONFIG-006/007 → CONFIG-008 → CONFIG-009

**Phase 6**:
12. EXPORT-001 → EXPORT-002/003/004/005 → EXPORT-006

**Phase 7**:
13. DSP-007/009/013 (parallel) → DSP-008 → DSP-010

**Phase 8**:
14. PERF-001, 003, 004 (parallel)

**Phase 9**:
15. TEST-006, 009
16. VIZ-017 (Composition engine - moonshot)
17. DOCS-005, 007, 010

---

## Story Creation Priority Order

**Batch 1 (Create first - Sacred Geometry)**:
- VIZ-013, VIZ-014, VIZ-015, VIZ-016

**Batch 2 (Effects foundation)**:
- EFFECTS-007, 001, 002, 003, 004, 005, 006, 008, 009

**Batch 3 (Color system)**:
- COLOR-001, 002, 003

**Batch 4 (Additional visualizers)**:
- VIZ-009, 010, 011, 018, 019, 020, 021

**Batch 5 (Controls & presets)**:
- CONTROL-001, 002
- CONFIG-005, 006, 007, 008, 009

**Batch 6 (Export)**:
- EXPORT-001, 002, 003, 004, 005, 006

**Batch 7 (DSP)**:
- DSP-007, 008, 009, 010, 013

**Batch 8 (Performance & Testing)**:
- PERF-001, 003, 004
- TEST-006, 009

**Batch 9 (Interaction & Composition)**:
- VIZ-012, VIZ-017

**Batch 10 (Documentation)**:
- DOCS-005, 007, 010

---

## Summary

- **Total stories to create**: ~50
- **All Phase 3-4 dependencies**: ✅ MET (can start immediately)
- **Recommended start**: Sacred Geometry (VIZ-013-016) for user excitement
- **Alternative start**: Effects Pipeline (EFFECTS-007+) for foundation
- **Fastest wins**: Color enhancement (COLOR-001-003)

**Ready to create the first batch of stories!**
