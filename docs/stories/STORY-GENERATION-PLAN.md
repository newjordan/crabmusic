# CrabMusic Story Generation Plan
## Comprehensive Expansion Roadmap from Brainstorming Session

**Created**: 2025-10-31
**Purpose**: Define all story files needed to implement features from brainstorming session
**Current Status**: Foundation Complete - Ready for Phase 3+ Expansion

---

## Implementation Status Summary

### ✅ Phase 1-2: Foundation & MVP (COMPLETE)
- Audio capture (CPAL/WASAPI) ✅
- DSP processing (FFT, frequency bands, smoothing) ✅
- Grid-based rendering system ✅
- Sine wave visualizer ✅
- Spectrum analyzer ✅
- Oscilloscope visualizer ✅
- Beat detection (energy-based) ✅
- Color support (6 color schemes) ✅
- Braille rendering system ✅
- Mode switching (1-5 keys) ✅
- Pipeline integration ✅
- Configuration system ✅

### 🎯 Next: Phase 3-5 Expansions

---

## Story Categories & Numbering Scheme

### Visualization (VIZ-009+)
- VIZ-009: Character Coverage Test Suite
- VIZ-010: Matrix Rain Effect
- VIZ-011: Cellular Automata Visualizer
- VIZ-012: Element Interaction Framework
- VIZ-013: Flower of Life Visualizer
- VIZ-014: Mandala Generator
- VIZ-015: Kaleidoscope Patterns
- VIZ-016: Recursive Pattern System
- VIZ-017: Generative Composition Engine
- VIZ-018: Lissajous Curves (Enhanced Oscilloscope)
- VIZ-019: Spiral Patterns
- VIZ-020: Circular Waves
- VIZ-021: Pulsing Blob Visualizer

### Effects & Post-Processing (EFFECTS-001+)
- EFFECTS-001: Perlin Noise Layer
- EFFECTS-002: Turbulence Field Generator
- EFFECTS-003: Bloom Effect
- EFFECTS-004: Scanline Effect
- EFFECTS-005: CRT Curve & Distortion
- EFFECTS-006: Phosphor Glow Effect
- EFFECTS-007: Effect Pipeline Framework
- EFFECTS-008: Effect Parameter Modulation
- EFFECTS-009: Effect Compositing System

### Advanced DSP (DSP-007+)
- DSP-007: Harmonic Analysis
- DSP-008: Pitch Detection
- DSP-009: Spectral Centroid
- DSP-010: Audio Feature Extraction Framework
- DSP-011: Genre Detection (ML-based, optional)
- DSP-012: Dynamic Range Compression
- DSP-013: Transient Detection

### Color & Theming (COLOR-001+)
- COLOR-001: Advanced Color Schemes (CGA, EGA, VGA palettes)
- COLOR-002: Dynamic Gradient Generator
- COLOR-003: Frequency-to-Color Mapping System
- COLOR-004: Theme Manager
- COLOR-005: Color Animation System
- COLOR-006: User-Defined Color Palettes

### Interactive Controls (CONTROL-001+)
- CONTROL-001: Keyboard Control System
- CONTROL-002: Real-Time Parameter Adjustment
- CONTROL-003: Preset Quick-Switch
- CONTROL-004: Visual Mode Sequencer
- CONTROL-005: Recording Control Interface
- CONTROL-006: MIDI Controller Integration (optional)

### Configuration & Presets (CONFIG-005+)
- CONFIG-005: Preset System Framework
- CONFIG-006: Genre-Specific Presets
- CONFIG-007: User Preset Manager
- CONFIG-008: Preset Import/Export
- CONFIG-009: Visual Preset Editor (TUI)

### Export & Recording (EXPORT-001+)
- EXPORT-001: Frame Capture System
- EXPORT-002: ASCII Animation Recorder
- EXPORT-003: Video Export (with FFmpeg)
- EXPORT-004: Screenshot Feature
- EXPORT-005: GIF Export
- EXPORT-006: Playback Mode (replay recordings)

### Performance & Optimization (PERF-001+)
- PERF-001: Rendering Performance Profiling
- PERF-002: GPU-Accelerated Text Rendering (optional)
- PERF-003: Multi-threaded Visual Processing
- PERF-004: Memory Pool Optimization
- PERF-005: Lazy Evaluation System

### Advanced Testing (TEST-006+)
- TEST-006: Visual Regression Testing
- TEST-007: Performance Benchmark Suite
- TEST-008: Audio Stress Testing
- TEST-009: Character Coverage Validation Tests
- TEST-010: Effect Quality Tests

### Documentation (DOCS-005+)
- DOCS-005: Visual Mode Gallery
- DOCS-006: Effect Showcase
- DOCS-007: Preset Creation Guide
- DOCS-008: Advanced Configuration Reference
- DOCS-009: Architecture Deep Dive
- DOCS-010: Sacred Geometry Mathematics Reference

---

## Phase 3: Enhanced Visuals & Effects

**Goal**: Expand visual vocabulary and add post-processing effects
**Duration**: 4-6 weeks
**Priority**: High

### Stories to Create:

#### Visualization Enhancements
1. **VIZ-009: Character Coverage Test Suite**
   - Epic: Visualization Engine
   - Priority: P1
   - Effort: 2-3 days
   - Description: Visual validation of character sets for different shapes
   - Dependencies: VIZ-001, VIZ-003
   - Features:
     - Static sine wave test
     - Amplitude variation test
     - Character palette comparison
     - 45° diagonal stress test
     - Gradient smoothness test

2. **VIZ-010: Matrix Rain Effect**
   - Epic: Visualization Engine
   - Priority: P2
   - Effort: 3-4 days
   - Description: Falling character streams with layering (only mode allowing overlap)
   - Dependencies: VIZ-004
   - Features:
     - Particle system architecture
     - Depth sorting/layering
     - Fade trails
     - Audio-driven spawn rate
     - Character randomization

3. **VIZ-011: Cellular Automata Visualizer**
   - Epic: Visualization Engine
   - Priority: P2
   - Effort: 4-5 days
   - Description: Conway's Game of Life with audio-driven cell spawning
   - Dependencies: VIZ-004
   - Features:
     - CA grid logic
     - Audio-to-rule mapping
     - State management
     - Multiple rule sets

4. **VIZ-021: Pulsing Blob Visualizer**
   - Epic: Visualization Engine
   - Priority: P2
   - Effort: 2-3 days
   - Description: Organic pulsing shapes driven by audio
   - Dependencies: VIZ-003, VIZ-004
   - Features:
     - Metaball-style rendering
     - Audio-driven pulse rate
     - Multiple blob interactions

#### Effects Framework
5. **EFFECTS-007: Effect Pipeline Framework**
   - Epic: Effects & Post-Processing
   - Priority: P1
   - Effort: 3-4 days
   - Description: Foundation for applying post-processing effects to visuals
   - Dependencies: VIZ-001, VIZ-004
   - Features:
     - Effect trait definition
     - Pipeline composition
     - Effect ordering
     - Parameter passing

6. **EFFECTS-001: Perlin Noise Layer**
   - Epic: Effects & Post-Processing
   - Priority: P2
   - Effort: 3-4 days
   - Description: Perlin noise as overlay effect
   - Dependencies: EFFECTS-007
   - Features:
     - Real-time noise generation
     - Octave control
     - Audio-driven intensity
     - Multiple noise types

7. **EFFECTS-002: Turbulence Field Generator**
   - Epic: Effects & Post-Processing
   - Priority: P2
   - Effort: 3-4 days
   - Description: Turbulent flow field effects
   - Dependencies: EFFECTS-001
   - Features:
     - Flow field calculation
     - Particle advection
     - Audio-driven turbulence

8. **EFFECTS-003: Bloom Effect**
   - Epic: Effects & Post-Processing
   - Priority: P2
   - Effort: 2-3 days
   - Description: Glow/bloom post-processing
   - Dependencies: EFFECTS-007
   - Features:
     - Brightness threshold
     - Blur radius control
     - Audio-reactive intensity
     - Color bloom

9. **EFFECTS-004: Scanline Effect**
   - Epic: Effects & Post-Processing
   - Priority: P2
   - Effort: 1-2 days
   - Description: CRT-style scanlines
   - Dependencies: EFFECTS-007
   - Features:
     - Horizontal scanlines
     - Intensity control
     - Optional animation

10. **EFFECTS-005: CRT Curve & Distortion**
    - Epic: Effects & Post-Processing
    - Priority: P2
    - Effort: 3-4 days
    - Description: Barrel distortion for CRT look
    - Dependencies: EFFECTS-007
    - Features:
      - Lens distortion
      - Corner vignetting
      - Screen curvature

11. **EFFECTS-006: Phosphor Glow Effect**
    - Epic: Effects & Post-Processing
    - Priority: P2
    - Effort: 2-3 days
    - Description: Phosphor persistence simulation
    - Dependencies: EFFECTS-007
    - Features:
      - Temporal persistence
      - Color decay
      - Glow intensity

#### Color System Expansion
12. **COLOR-001: Advanced Color Schemes**
    - Epic: Color & Theming
    - Priority: P2
    - Effort: 2-3 days
    - Description: Add CGA, EGA, VGA, and retro palettes
    - Dependencies: Current color system
    - Features:
      - CGA 16-color palette
      - EGA 64-color palette
      - VGA 256-color palette
      - C64 palette
      - ZX Spectrum palette

13. **COLOR-002: Dynamic Gradient Generator**
    - Epic: Color & Theming
    - Priority: P2
    - Effort: 3-4 days
    - Description: Procedural gradient generation
    - Dependencies: COLOR-001
    - Features:
      - HSV/RGB interpolation
      - Multi-stop gradients
      - Audio-driven transitions
      - Gradient presets

14. **COLOR-003: Frequency-to-Color Mapping System**
    - Epic: Color & Theming
    - Priority: P2
    - Effort: 3-4 days
    - Description: Map frequency bands to colors systematically
    - Dependencies: DSP-002, COLOR-002
    - Features:
      - Band-to-hue mapping
      - Amplitude-to-saturation
      - Configurable mappings
      - Multiple mapping modes

---

## Phase 4: Sacred Geometry Suite

**Goal**: Implement sacred geometry visualizations
**Duration**: 4-6 weeks
**Priority**: High (User excitement area)

### Stories to Create:

15. **VIZ-013: Flower of Life Visualizer**
    - Epic: Sacred Geometry
    - Priority: P1
    - Effort: 5-7 days
    - Description: Classic Flower of Life pattern with audio reactivity
    - Dependencies: VIZ-003, VIZ-004
    - Features:
      - Circular pattern generation
      - Multiple rings (configurable depth)
      - Audio-driven ring expansion
      - Rotation animation
      - Integration with existing sacred geometry work

16. **VIZ-014: Mandala Generator**
    - Epic: Sacred Geometry
    - Priority: P1
    - Effort: 5-7 days
    - Description: Procedural mandala generation
    - Dependencies: VIZ-003, VIZ-004
    - Features:
      - Radial symmetry (4, 6, 8, 12-fold)
      - Pattern layers
      - Audio-driven complexity
      - Rotation and pulsing
      - Template system

17. **VIZ-015: Kaleidoscope Patterns**
    - Epic: Sacred Geometry
    - Priority: P1
    - Effort: 4-6 days
    - Description: Kaleidoscopic symmetry visualizations
    - Dependencies: VIZ-003, VIZ-004
    - Features:
      - Mirror symmetry
      - Configurable sector count
      - Audio-driven pattern evolution
      - Color cycling
      - Rotation modes

18. **VIZ-016: Recursive Pattern System**
    - Epic: Sacred Geometry
    - Priority: P2
    - Effort: 6-8 days
    - Description: Framework for recursively nested patterns
    - Dependencies: VIZ-013, VIZ-014, VIZ-015
    - Features:
      - Recursive depth management
      - Parent-child pattern relationships
      - Audio propagation through levels
      - Performance optimization
      - Fractal-like behaviors

---

## Phase 5: Interaction & Composition

**Goal**: Enable element interaction and dynamic composition
**Duration**: 3-4 weeks
**Priority**: Medium

### Stories to Create:

19. **VIZ-012: Element Interaction Framework**
    - Epic: Visualization Engine
    - Priority: P1
    - Effort: 5-7 days
    - Description: System for visual elements to interact
    - Dependencies: VIZ-004, EFFECTS-007
    - Features:
      - Layering system
      - Morphing/transitions
      - Nesting capability
      - Reactive triggers
      - Modulation routing

20. **VIZ-017: Generative Composition Engine**
    - Epic: Visualization Engine
    - Priority: P2
    - Effort: 7-10 days
    - Description: AI/algorithmic composition of visuals
    - Dependencies: VIZ-012, all visualizers
    - Features:
      - Composition rules
      - Genre detection integration
      - Aesthetic evaluation
      - Automatic mode switching
      - Learning/adaptation (optional)

21. **EFFECTS-008: Effect Parameter Modulation**
    - Epic: Effects & Post-Processing
    - Priority: P2
    - Effort: 3-4 days
    - Description: Route audio parameters to effect controls
    - Dependencies: EFFECTS-007, DSP-002
    - Features:
      - Modulation routing matrix
      - Scaling/mapping
      - LFO modulation
      - Envelope followers

22. **EFFECTS-009: Effect Compositing System**
    - Epic: Effects & Post-Processing
    - Priority: P2
    - Effort: 4-5 days
    - Description: Blend multiple effects with blend modes
    - Dependencies: EFFECTS-007, EFFECTS-008
    - Features:
      - Blend modes (add, multiply, overlay)
      - Effect chains
      - Parallel processing
      - Effect presets

---

## Phase 6: Advanced Controls & Presets

**Goal**: Interactive control and preset management
**Duration**: 3-4 weeks
**Priority**: Medium

### Stories to Create:

23. **CONTROL-001: Keyboard Control System**
    - Epic: Interactive Controls
    - Priority: P1
    - Effort: 3-4 days
    - Description: Comprehensive keyboard shortcuts
    - Dependencies: Pipeline
    - Features:
      - Mode switching (extended beyond 1-5)
      - Parameter adjustment
      - Effect toggles
      - Help overlay

24. **CONTROL-002: Real-Time Parameter Adjustment**
    - Epic: Interactive Controls
    - Priority: P1
    - Effort: 3-4 days
    - Description: Live parameter tweaking with visual feedback
    - Dependencies: CONTROL-001, CONFIG-003
    - Features:
      - On-screen sliders (TUI)
      - Value display
      - Parameter grouping
      - Save current state

25. **CONFIG-005: Preset System Framework**
    - Epic: Configuration & Presets
    - Priority: P1
    - Effort: 4-5 days
    - Description: Save/load complete visual configurations
    - Dependencies: CONFIG-001, CONFIG-002
    - Features:
      - Preset storage format
      - Quick-load system
      - Preset metadata
      - Preset validation

26. **CONFIG-006: Genre-Specific Presets**
    - Epic: Configuration & Presets
    - Priority: P2
    - Effort: 3-4 days
    - Description: Pre-configured presets for music genres
    - Dependencies: CONFIG-005
    - Features:
      - Electronic/EDM preset
      - Classical preset
      - Rock/Metal preset
      - Jazz preset
      - Hip-Hop preset
      - Ambient preset

27. **CONFIG-009: Visual Preset Editor (TUI)**
    - Epic: Configuration & Presets
    - Priority: P2
    - Effort: 5-7 days
    - Description: In-app preset editor
    - Dependencies: CONFIG-005, CONTROL-002
    - Features:
      - TUI interface
      - Live preview
      - Parameter organization
      - Save/load workflow

---

## Phase 7: Recording & Export

**Goal**: Capture and export visualizations
**Duration**: 2-3 weeks
**Priority**: Medium

### Stories to Create:

28. **EXPORT-001: Frame Capture System**
    - Epic: Export & Recording
    - Priority: P1
    - Effort: 3-4 days
    - Description: Capture grid buffer to memory
    - Dependencies: VIZ-001
    - Features:
      - Frame buffer capture
      - Timestamping
      - Memory management
      - Buffer pool

29. **EXPORT-002: ASCII Animation Recorder**
    - Epic: Export & Recording
    - Priority: P1
    - Effort: 3-4 days
    - Description: Record to .txt or .anim format
    - Dependencies: EXPORT-001
    - Features:
      - Text file output
      - Frame timing data
      - Compression options
      - Metadata

30. **EXPORT-003: Video Export (with FFmpeg)**
    - Epic: Export & Recording
    - Priority: P2
    - Effort: 5-7 days
    - Description: Export as MP4/WebM video
    - Dependencies: EXPORT-001
    - Features:
      - FFmpeg integration
      - Format selection
      - Quality settings
      - Audio synchronization

31. **EXPORT-004: Screenshot Feature**
    - Epic: Export & Recording
    - Priority: P2
    - Effort: 1-2 days
    - Description: Single-frame capture
    - Dependencies: EXPORT-001
    - Features:
      - PNG export
      - Configurable resolution
      - Keyboard shortcut
      - Filename generation

32. **EXPORT-005: GIF Export**
    - Epic: Export & Recording
    - Priority: P2
    - Effort: 2-3 days
    - Description: Export as animated GIF
    - Dependencies: EXPORT-001
    - Features:
      - GIF encoding
      - Frame rate control
      - Loop count
      - Color palette optimization

33. **EXPORT-006: Playback Mode**
    - Epic: Export & Recording
    - Priority: P2
    - Effort: 3-4 days
    - Description: Replay recorded animations
    - Dependencies: EXPORT-002
    - Features:
      - Load recorded animations
      - Playback controls
      - Speed adjustment
      - No audio dependency

---

## Phase 8: Advanced DSP & Analysis

**Goal**: Enhanced audio analysis capabilities
**Duration**: 3-4 weeks
**Priority**: Low-Medium

### Stories to Create:

34. **DSP-007: Harmonic Analysis**
    - Epic: DSP Processing
    - Priority: P2
    - Effort: 4-5 days
    - Description: Detect harmonic content
    - Dependencies: DSP-001, DSP-002
    - Features:
      - Harmonic peak detection
      - Fundamental frequency
      - Harmonic-to-noise ratio
      - Timbre analysis

35. **DSP-008: Pitch Detection**
    - Epic: DSP Processing
    - Priority: P2
    - Effort: 4-5 days
    - Description: Real-time pitch tracking
    - Dependencies: DSP-007
    - Features:
      - Autocorrelation method
      - MIDI note output
      - Pitch stability
      - Polyphonic support (optional)

36. **DSP-009: Spectral Centroid**
    - Epic: DSP Processing
    - Priority: P2
    - Effort: 2-3 days
    - Description: Calculate spectral "brightness"
    - Dependencies: DSP-001
    - Features:
      - Centroid calculation
      - Temporal tracking
      - Visual mapping

37. **DSP-010: Audio Feature Extraction Framework**
    - Epic: DSP Processing
    - Priority: P2
    - Effort: 5-6 days
    - Description: Extensible feature extraction system
    - Dependencies: DSP-001 through DSP-009
    - Features:
      - Feature plugin architecture
      - Feature aggregation
      - Real-time computation
      - Feature history

38. **DSP-013: Transient Detection**
    - Epic: DSP Processing
    - Priority: P2
    - Effort: 3-4 days
    - Description: Detect attack transients
    - Dependencies: DSP-001
    - Features:
      - High-frequency transients
      - Percussion detection
      - Visual trigger events

---

## Phase 9: Performance & Optimization

**Goal**: Optimize for higher resolutions and effects
**Duration**: 2-3 weeks
**Priority**: Medium

### Stories to Create:

39. **PERF-001: Rendering Performance Profiling**
    - Epic: Performance & Optimization
    - Priority: P1
    - Effort: 2-3 days
    - Description: Comprehensive performance metrics
    - Dependencies: Pipeline
    - Features:
      - Frame timing
      - Component profiling
      - Bottleneck identification
      - Performance dashboard

40. **PERF-003: Multi-threaded Visual Processing**
    - Epic: Performance & Optimization
    - Priority: P2
    - Effort: 5-7 days
    - Description: Parallelize visual calculations
    - Dependencies: VIZ-004
    - Features:
      - Work stealing scheduler
      - Grid subdivision
      - Lock-free updates
      - Thread pool management

41. **PERF-004: Memory Pool Optimization**
    - Epic: Performance & Optimization
    - Priority: P2
    - Effort: 3-4 days
    - Description: Reduce allocations with memory pools
    - Dependencies: VIZ-001
    - Features:
      - Object pooling
      - Arena allocators
      - Zero-copy optimizations

---

## Phase 10: Enhanced Testing & Documentation

**Goal**: Comprehensive testing and documentation
**Duration**: 2-3 weeks
**Priority**: Medium

### Stories to Create:

42. **TEST-006: Visual Regression Testing**
    - Epic: Testing & Validation
    - Priority: P1
    - Effort: 4-5 days
    - Description: Automated visual comparison
    - Dependencies: All visualizers
    - Features:
      - Reference frame capture
      - Pixel-perfect comparison
      - Diff visualization
      - CI integration

43. **TEST-009: Character Coverage Validation Tests**
    - Epic: Testing & Validation
    - Priority: P2
    - Effort: 2-3 days
    - Description: Automated coverage algorithm tests
    - Dependencies: VIZ-003
    - Features:
      - Known shape tests
      - Coverage accuracy metrics
      - Performance benchmarks

44. **DOCS-005: Visual Mode Gallery**
    - Epic: Documentation & Release
    - Priority: P2
    - Effort: 2-3 days
    - Description: Showcase of all visual modes
    - Dependencies: All visualizers
    - Features:
      - Screenshots/recordings
      - Mode descriptions
      - Parameter examples
      - Use case suggestions

45. **DOCS-010: Sacred Geometry Mathematics Reference**
    - Epic: Documentation & Release
    - Priority: P2
    - Effort: 3-4 days
    - Description: Mathematical foundations of sacred geometry
    - Dependencies: VIZ-013, VIZ-014, VIZ-015
    - Features:
      - Algorithm explanations
      - Mathematical proofs
      - Historical context
      - Implementation notes

---

## Additional Visualizer Ideas (Lower Priority)

46. **VIZ-018: Enhanced Lissajous Curves**
    - 3D-style Lissajous with depth
    - Multiple frequency ratios
    - Color by phase

47. **VIZ-019: Spiral Patterns**
    - Archimedean spirals
    - Logarithmic spirals
    - Audio-driven rotation

48. **VIZ-020: Circular Waves**
    - Concentric ripples
    - Wave interference
    - Bass-driven pulse

49. **VIZ-022: Particle Systems**
    - Physics-based particles
    - Audio-driven forces
    - Multiple emitter types

50. **VIZ-023: Tunnel Effect**
    - Classic demo scene effect
    - Depth illusion
    - Texture mapping

---

## Optional / Experimental Features

51. **CONTROL-006: MIDI Controller Integration**
    - Epic: Interactive Controls
    - Priority: P3
    - Effort: 5-7 days
    - Description: Control via MIDI hardware

52. **DSP-011: Genre Detection (ML-based)**
    - Epic: DSP Processing
    - Priority: P3
    - Effort: 10-14 days
    - Description: Automatic music genre classification

53. **PERF-002: GPU-Accelerated Text Rendering**
    - Epic: Performance & Optimization
    - Priority: P3
    - Effort: 7-10 days
    - Description: GPU acceleration for text rendering

---

## Story Creation Workflow

### Step 1: Prioritize Stories
Review the above list and assign final priorities based on:
- User excitement (sacred geometry is high!)
- Technical dependencies
- Estimated impact
- Development effort

### Step 2: Create Story Files in Batches
Organize story creation into batches of 5-10 stories:

**Batch 1: Effects Foundation** (1-2 days)
- EFFECTS-007: Effect Pipeline Framework
- EFFECTS-001: Perlin Noise Layer
- EFFECTS-003: Bloom Effect
- EFFECTS-004: Scanline Effect
- EFFECTS-006: Phosphor Glow Effect

**Batch 2: Sacred Geometry** (2-3 days)
- VIZ-013: Flower of Life Visualizer
- VIZ-014: Mandala Generator
- VIZ-015: Kaleidoscope Patterns
- VIZ-016: Recursive Pattern System

**Batch 3: Color System** (1 day)
- COLOR-001: Advanced Color Schemes
- COLOR-002: Dynamic Gradient Generator
- COLOR-003: Frequency-to-Color Mapping

**Batch 4: Additional Visualizers** (1-2 days)
- VIZ-009: Character Coverage Test Suite
- VIZ-010: Matrix Rain Effect
- VIZ-011: Cellular Automata Visualizer
- VIZ-021: Pulsing Blob Visualizer

**Batch 5: Interaction & Composition** (1-2 days)
- VIZ-012: Element Interaction Framework
- VIZ-017: Generative Composition Engine
- EFFECTS-008: Effect Parameter Modulation
- EFFECTS-009: Effect Compositing System

**Batch 6: Controls & Presets** (1-2 days)
- CONTROL-001: Keyboard Control System
- CONTROL-002: Real-Time Parameter Adjustment
- CONFIG-005: Preset System Framework
- CONFIG-006: Genre-Specific Presets

**Batch 7: Export & Recording** (1-2 days)
- EXPORT-001: Frame Capture System
- EXPORT-002: ASCII Animation Recorder
- EXPORT-003: Video Export
- EXPORT-004: Screenshot Feature
- EXPORT-005: GIF Export

**Batch 8: Advanced DSP** (1-2 days)
- DSP-007: Harmonic Analysis
- DSP-008: Pitch Detection
- DSP-009: Spectral Centroid
- DSP-013: Transient Detection

**Batch 9: Performance & Testing** (1 day)
- PERF-001: Rendering Performance Profiling
- PERF-003: Multi-threaded Visual Processing
- TEST-006: Visual Regression Testing
- TEST-009: Character Coverage Validation

**Batch 10: Documentation** (1 day)
- DOCS-005: Visual Mode Gallery
- DOCS-007: Preset Creation Guide
- DOCS-010: Sacred Geometry Mathematics Reference

### Step 3: Story Template Usage
Each story should follow this template:

```markdown
# [STORY-ID]: [Title]

**Epic**: [Epic Name]
**Priority**: P0/P1/P2/P3
**Estimated Effort**: X days
**Status**: Not Started

## Description

[What needs to be built and why - 2-3 paragraphs]

## Acceptance Criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3
[5-10 specific, testable criteria]

## Technical Approach

[High-level implementation approach]

**Key Components**:
- Component 1: Description
- Component 2: Description

**Algorithm/Architecture**:
[Detailed technical approach]

**Code Structure**:
```rust
// Pseudocode or structure example
```

## Dependencies

**Depends on**:
- STORY-ID: Description

**Blocks**:
- STORY-ID: Description

## Architecture References

- docs/architecture.md: [Relevant section]
- docs/brainstorming-session-results.md: [Line numbers]
- [External references if applicable]

## Testing Requirements

**Unit Tests**:
- Test 1
- Test 2

**Integration Tests**:
- Test 1
- Test 2

**Manual Testing**:
- Test procedure 1
- Test procedure 2

## Notes for AI Agent

- Implementation guidance
- Gotchas to watch for
- Performance considerations
- Suggested approach

## Future Enhancements

- Potential expansion 1
- Potential expansion 2
```

### Step 4: Implementation Order

**Recommended implementation sequence after story creation**:

1. **Phase 3a: Effects Foundation** (Start here)
   - EFFECTS-007 → EFFECTS-001 → EFFECTS-003/004/006

2. **Phase 3b: Color Enhancement**
   - COLOR-001 → COLOR-002 → COLOR-003

3. **Phase 4: Sacred Geometry** (High user interest!)
   - VIZ-013 → VIZ-014 → VIZ-015 → VIZ-016

4. **Phase 5: Interaction**
   - VIZ-012 → EFFECTS-008 → EFFECTS-009

5. **Phase 6: Controls & Presets**
   - CONTROL-001 → CONTROL-002 → CONFIG-005 → CONFIG-006

6. **Phases 7-10**: As needed based on priorities

---

## Summary

**Total New Stories to Create**: ~45-50 stories
**Estimated Story Creation Time**: 10-15 days (if done in batches)
**Estimated Implementation Time**: 20-30 weeks (parallel work possible)

**Next Steps**:
1. Review this plan and adjust priorities
2. Choose first batch to create (recommend: Sacred Geometry for user excitement)
3. Create 5-10 story files at a time
4. Begin implementation following dependencies

---

**Ready to start creating stories? Let me know which batch you'd like to tackle first!**
