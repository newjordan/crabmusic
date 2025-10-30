● 🚀 CrabMusic: Parallel Agent Execution Guide

  Status: Ready to Execute
  Estimated Time to MVP: 3-4 weeks with parallel agent work
  Last Updated: 2025-10-29

  ---
  📋 Pre-Flight Checklist

  Before starting agent work:

  - ✅ Architecture document complete (docs/architecture.md)
  - ✅ Architecture sharded (docs/architecture/*.md)
  - ✅ Implementation plan created (docs/implementation-plan.md)
  - ✅ Story templates created (5 samples in docs/stories/)
  - ✅ Core config validated (.bmad-core/core-config.yaml)
  - 🟡 Remaining stories need creation (~35 more)

  ---
  🎯 Execution Strategy: Two Parallel Tracks

  Track A: Complete Story Backlog (Parallel with Track B)

  Goal: Fill in all remaining story files
  Duration: 4-6 hours
  Agent Type: Analyst or Architect

  Track B: Begin MVP Implementation (Parallel with Track A)

  Goal: Start implementing critical path stories
  Duration: 2-3 weeks
  Agent Type: Dev agents

  Why Parallel? The 5 scaffold stories are complete enough to
  start implementation on critical path while remaining stories
  are being completed.

  ---
  🔄 Phase 1: Story Backlog Completion (Optional Start)

  Goal: Create all remaining story files following the template
  pattern

  Step 1.1: Identify Missing Stories

  From docs/implementation-plan.md, these stories need creation:

  Foundation (1 remaining):
  - FOUND-002 - CI/CD Pipeline Setup

  Audio Capture (2 remaining):
  - AUDIO-001 - Define Audio Capture Interface
  - AUDIO-003 - Ring Buffer Implementation
  - AUDIO-004 - Audio Testing & Validation

  DSP Processing (4 remaining):
  - DSP-002 - Frequency Band Extraction
  - DSP-003 - Parameter Smoothing
  - DSP-004 - Beat Detection
  - DSP-005 - DSP Testing

  Visualization (6 remaining):
  - VIZ-001 - Grid Buffer Data Structure
  - VIZ-002 - Character Set Definitions
  - VIZ-003 - Character Coverage Algorithm
  - VIZ-004 - Visualizer Trait Design
  - VIZ-006 - Spectrum Visualizer (post-MVP)
  - VIZ-007 - Oscilloscope Visualizer (post-MVP)

  Rendering (4 remaining):
  - RENDER-001 - Terminal Initialization
  - RENDER-002 - Ratatui Integration
  - RENDER-003 - Differential Rendering
  - RENDER-004 - Terminal Resize Handling

  Configuration (4 remaining):
  - CONFIG-001 - Config Structure
  - CONFIG-002 - YAML Loading
  - CONFIG-003 - Hot Reload
  - CONFIG-004 - Default Config

  Pipeline (4 remaining):
  - PIPELINE-002 - Thread Coordination
  - PIPELINE-003 - Error Handling
  - PIPELINE-004 - CLI Interface
  - PIPELINE-005 - Logging Setup

  Testing (5 remaining):
  - TEST-001 through TEST-005

  Documentation (5 remaining):
  - DOCS-001 through DOCS-004
  - RELEASE-001

  Total: ~35 stories

  ---
  Step 1.2: Parallel Story Generation

  Command Template for Agent:

  I need you to create implementation stories for crabmusic
  following the established template pattern.

  Context files to load:
  - docs/implementation-plan.md (for story details)
  - docs/architecture.md (for technical context)
  - docs/stories/FOUND-001-project-setup.md (for template
  structure)
  - docs/stories/VIZ-005-sine-wave-visualizer.md (for quality
  example)

  Create the following stories:
  [List 5-10 story IDs from the list above]

  For each story, include:
  1. Description (what needs to be built and why)
  2. Acceptance Criteria (specific, testable)
  3. Technical Approach (with code examples where helpful)
  4. Dependencies (from implementation plan)
  5. Architecture References (link to relevant sections)
  6. Testing Requirements (unit, integration, manual)
  7. Notes for AI Agent (implementation guidance)

  Output each story as a separate file:
  docs/stories/[STORY-ID]-[kebab-case-name].md

  Batch Strategy: Create stories in groups of 5-10 to avoid
  overwhelming single agent session.

  Run 3-4 Agents in Parallel (separate terminal sessions):
  - Agent 1: AUDIO-* stories
  - Agent 2: DSP-* stories
  - Agent 3: VIZ-* stories
  - Agent 4: RENDER-* + CONFIG-* stories

  ---
  🚀 Phase 2: MVP Critical Path (START HERE!)

  Goal: Implement minimum viable product - sine wave reacts to
  audio

  Critical Path Dependencies

  FOUND-001 (Project Setup)
      ↓
  AUDIO-001 (Audio Interface) + AUDIO-003 (Ring Buffer)
      ↓
  AUDIO-002 (CPAL Implementation)
      ↓
  DSP-001 (FFT Processor)
      ↓
  DSP-002 (Frequency Bands)
      ↓
  VIZ-001 (Grid Buffer) + VIZ-003 (Coverage) + VIZ-004 (Trait)
      ↓
  VIZ-005 (Sine Wave Visualizer) ⭐ MVP CORE
      ↓
  RENDER-001 (Terminal Init) + RENDER-002 (Ratatui)
      ↓
  PIPELINE-001 (Main Loop) 🎉 MVP INTEGRATION
      ↓
  TEST-005 (Audio Validation)

  ---
  Step 2.1: Foundation (Week 1, Day 1-2)

  Stories: FOUND-001

  Agent Command:
  /BMad:agents:dev

  Load these files:
  - docs/stories/FOUND-001-project-setup.md
  - docs/architecture/tech-stack.md
  - docs/architecture/source-tree.md
  - docs/architecture/coding-standards.md

  Task: Implement FOUND-001 - Project Setup and Scaffolding

  Follow the acceptance criteria exactly. Create the complete
  Cargo.toml with all dependencies from tech-stack.md, create
  the source tree structure, and add placeholder modules.

  When complete, verify:
  - cargo build compiles
  - cargo test runs
  - cargo clippy passes
  - Project structure matches source-tree.md

  Expected Duration: 1-2 hours
  Output: Complete Rust project scaffold

  ---
  Step 2.2: Parallel Stream A - Audio Pipeline (Week 1, Day 2-5)

  Stories: AUDIO-001 → AUDIO-003 → AUDIO-002 → AUDIO-004

  Agent 1 Session:
  /BMad:agents:dev

  Context: We're building the audio capture system for crabmusic

  Load:
  - docs/stories/AUDIO-001-audio-capture-trait.md (needs
  creation first, or improvise from architecture)
  - docs/architecture.md (Audio Capture Component section)
  - docs/architecture/coding-standards.md

  Task: Define the AudioCaptureDevice trait and AudioBuffer
  struct

  Create:
  - src/audio/capture.rs with AudioCaptureDevice trait
  - src/audio/buffer.rs with AudioBuffer struct
  - Unit tests for AudioBuffer

  Mark story AUDIO-001 as complete when done.

  Agent 2 Session (parallel):
  /BMad:agents:dev

  Load:
  - docs/stories/AUDIO-003-ring-buffer.md (or improvise from
  architecture)
  - docs/architecture.md (Ring Buffer section)

  Task: Implement lock-free ring buffer for audio pipeline

  Use ringbuf crate or crossbeam. Must support:
  - Single producer (audio thread)
  - Single consumer (main thread)
  - Non-blocking reads/writes

  Test with synthetic audio data.

  After AUDIO-001 & AUDIO-003 Complete:

  Agent 3 Session:
  /BMad:agents:dev

  Dependencies complete: AUDIO-001 ✅ AUDIO-003 ✅

  Load:
  - docs/stories/AUDIO-002-cpal-implementation.md
  - docs/architecture.md (Audio Capture Component)

  Task: Implement CPAL audio capture

  This is critical - take time to get it right. Test with real
  system audio.

  CRITICAL: Audio callback must never block. Use ring buffer
  from AUDIO-003.

  ---
  Step 2.3: Parallel Stream B - DSP Processing (Week 1-2)

  Stories: DSP-001 → DSP-002

  After AUDIO-003 complete, start DSP work:

  Agent Session:
  /BMad:agents:dev

  Dependencies: AUDIO-003 ✅ (need AudioBuffer)

  Load:
  - docs/stories/DSP-001-fft-processor.md
  - docs/architecture.md (DSP Processing Component)

  Task: Implement FFT processor with rustfft

  Must include:
  - Hann windowing
  - FFT computation
  - Magnitude spectrum extraction
  - Normalization

  PERFORMANCE CRITICAL: Must complete in <5ms

  Create benchmarks with criterion to verify performance.

  Then:
  /BMad:agents:dev

  Dependencies: DSP-001 ✅

  Load:
  - docs/stories/DSP-002-frequency-bands.md (or architecture)

  Task: Extract frequency bands from FFT spectrum

  Bands: bass, low-mid, mid, high-mid, treble
  Output: AudioParameters struct with all band values

  Test with synthetic sine waves at known frequencies.

  ---
  Step 2.4: Parallel Stream C - Visualization (Week 2)

  Stories: VIZ-001 → VIZ-003 → VIZ-004 → VIZ-005

  These can start early (don't need audio/DSP complete):

  Agent 1:
  /BMad:agents:dev

  Load:
  - docs/stories/VIZ-001-grid-buffer.md (or architecture)
  - docs/architecture.md (Visualization Engine)

  Task: Implement GridBuffer data structure

  Simple 2D array of characters with get/set methods.
  Pre-allocate to terminal dimensions.

  Agent 2 (parallel):
  /BMad:agents:dev

  Load:
  - docs/stories/VIZ-003-coverage-algorithm.md
  - docs/architecture.md (Visualization - Coverage section)

  Task: Implement character coverage calculation

  Algorithm that determines what percentage of a grid cell is
  covered by a shape.
  This is the secret sauce of the rendering system!

  Include anti-aliasing for smooth visuals.

  Agent 3 (parallel):
  /BMad:agents:dev

  Load:
  - docs/stories/VIZ-004-visualizer-trait.md
  - docs/architecture.md (Visualizer trait)

  Task: Define the Visualizer trait

  Must support:
  - update(&mut self, params: &AudioParameters)
  - render(&self, grid: &mut GridBuffer)
  - configure(&mut self, config: &VisualizerConfig)

  Create trait and test with mock implementation.

  After VIZ-001, VIZ-003, VIZ-004 complete:

  THE BIG ONE - Agent Session:
  /BMad:agents:dev

  Dependencies: VIZ-001 ✅ VIZ-003 ✅ VIZ-004 ✅ DSP-002 ✅

  Load:
  - docs/stories/VIZ-005-sine-wave-visualizer.md ⭐
  - docs/architecture.md (Sine Wave Visualizer)

  Task: Implement SineWaveVisualizer - THE MVP CORE

  This is the heart of the MVP. Take your time!

  Success criteria:
  - Sine wave renders smoothly
  - Reacts to audio parameters naturally
  - No jitter or lag
  - Visually satisfying

  Test with static parameters first, then integrate audio.

  🎯 THIS IS THE MOMENT OF TRUTH 🎯

  ---
  Step 2.5: Terminal Rendering (Week 2-3)

  Stories: RENDER-001 → RENDER-002

  Agent Session:
  /BMad:agents:dev

  Dependencies: VIZ-001 ✅ (need GridBuffer)

  Load:
  - docs/stories/RENDER-001-terminal-init.md (or architecture)
  - docs/stories/RENDER-002-ratatui-integration.md
  - docs/architecture.md (Terminal Renderer)

  Task: Implement terminal rendering with ratatui + crossterm

  Must:
  - Enter alternate screen
  - Handle raw mode
  - Render GridBuffer to display
  - Cleanup on exit (CRITICAL!)

  Test: Can render static grid without flicker.

  ---
  Step 2.6: Pipeline Integration (Week 3) 🎉

  Story: PIPELINE-001

  ALL DEPENDENCIES MUST BE COMPLETE before this step!

  Agent Session:
  /BMad:agents:dev

  🚨 INTEGRATION TIME 🚨

  Dependencies:
  - FOUND-001 ✅
  - AUDIO-002 ✅ AUDIO-003 ✅
  - DSP-001 ✅ DSP-002 ✅
  - VIZ-005 ✅
  - RENDER-002 ✅

  Load:
  - docs/stories/PIPELINE-001-main-loop.md
  - docs/architecture.md (Core Workflows sequence diagram)

  Task: Integrate all components into functioning application

  This brings everything together:
  - Main loop at 60 FPS
  - Audio → DSP → Visualization → Rendering pipeline
  - Graceful startup/shutdown
  - Error handling

  Success: Run app, play music, see smooth audio-reactive sine
  wave!

  🎉 MVP COMPLETE 🎉

  ---
  📊 Tracking Progress

  Create a Status Board

  In a file: PROJECT-STATUS.md

  # CrabMusic Implementation Status

  Last Updated: 2025-10-29

  ## Critical Path (MVP)

  - [ ] FOUND-001 - Project Setup
  - [ ] AUDIO-001 - Audio Interface
  - [ ] AUDIO-003 - Ring Buffer
  - [ ] AUDIO-002 - CPAL Implementation
  - [ ] DSP-001 - FFT Processor
  - [ ] DSP-002 - Frequency Bands
  - [ ] VIZ-001 - Grid Buffer
  - [ ] VIZ-003 - Coverage Algorithm
  - [ ] VIZ-004 - Visualizer Trait
  - [ ] VIZ-005 - Sine Wave Visualizer ⭐
  - [ ] RENDER-001 - Terminal Init
  - [ ] RENDER-002 - Ratatui Integration
  - [ ] PIPELINE-001 - Main Loop 🎉

  ## Supporting Stories

  [List all other stories with checkboxes]

  ## Current Blockers

  [List any blockers here]

  ## Next Up

  [What to work on next]

  Update this file after each story completes!

  ---
  🎯 Best Practices for Agent Parallelization

  ✅ DO:

  1. Check dependencies before starting a story
    - Look at "Depends on" section in story file
    - Don't start until dependencies are marked complete
  2. Load exactly what the story specifies
    - Story files list required architecture references
    - This keeps context focused and efficient
  3. Run independent stories in parallel
    - AUDIO work can parallel with VIZ work (different
  subsystems)
    - Multiple VIZ stories can parallel (VIZ-001, VIZ-003,
  VIZ-004)
  4. Update story status immediately after completion
    - Edit story file: Change Status: Not Started → Status:
  Complete
    - Update PROJECT-STATUS.md
  5. Commit after each story completes
  git add .
  git commit -m "Implement STORY-ID: Brief description"
  6. Test before marking complete
    - Run tests: cargo test
    - Check linting: cargo clippy
    - Verify acceptance criteria

  ❌ DON'T:

  1. Don't start a story with unmet dependencies
    - You'll waste time and create broken code
  2. Don't have multiple agents editing the same file
    - Coordinate file ownership to avoid merge conflicts
  3. Don't skip testing requirements
    - Every story has testing requirements - follow them!
  4. Don't parallelize dependent stories
    - DSP-002 depends on DSP-001 - must be sequential
  5. Don't accumulate uncommitted work
    - Commit after each story to track progress
  6. Don't ignore coding standards
    - Run cargo fmt and cargo clippy regularly

  ---
  🔄 Recommended Agent Session Pattern

  Session Start:

  1. Check PROJECT-STATUS.md for next available story
  2. Verify dependencies are complete
  3. Open story file to review requirements
  4. Start agent with story file loaded

  During Session:

  5. Implement per acceptance criteria
  6. Write tests (unit + integration as specified)
  7. Run cargo test and cargo clippy
  8. Validate against acceptance criteria

  Session End:

  9. Update story status to "Complete"
  10. Update PROJECT-STATUS.md
  11. Commit with meaningful message
  12. Identify next story for next session

  ---
  🎪 Example Parallel Execution Schedule

  Week 1: Foundation + Audio

  Monday AM:
  - Agent 1: FOUND-001 (project setup)

  Monday PM (after FOUND-001):
  - Agent 1: AUDIO-001 (trait definition)
  - Agent 2: AUDIO-003 (ring buffer) [parallel]
  - Agent 3: VIZ-001 (grid buffer) [parallel - no audio
  dependency]

  Tuesday (after AUDIO-001 & AUDIO-003):
  - Agent 1: AUDIO-002 (CPAL implementation)
  - Agent 2: VIZ-003 (coverage algorithm) [parallel]
  - Agent 3: VIZ-004 (visualizer trait) [parallel]

  Wednesday (after AUDIO-002):
  - Agent 1: DSP-001 (FFT processor)
  - Agent 2: RENDER-001 + RENDER-002 (terminal rendering)
  [parallel]

  Thursday:
  - Agent 1: DSP-002 (frequency bands)
  - Agent 2: Continue RENDER-002 if not done

  Friday:
  - Agent 1: VIZ-005 (sine wave visualizer) [needs DSP-002 ✅]

  ---
  Week 2: Integration

  Monday-Tuesday:
  - Agent 1: Continue VIZ-005 tuning (get it perfect!)
  - Agent 2: CONFIG-001, CONFIG-002 (config system) [parallel]

  Wednesday-Thursday:
  - Agent 1: PIPELINE-001 (main loop integration) 🎯

  Friday:
  - TEST-005 (manual testing with real music)
  - 🎉 MVP DEMO DAY 🎉

  ---
  Week 3+: Polish & Post-MVP

  Continue with testing, documentation, and post-MVP features
  (spectrum analyzer, oscilloscope, etc.)

  ---
  🚨 Common Pitfalls & Solutions

  Pitfall 1: "Agent doesn't have context"

  Solution: Always load the story file + relevant architecture
  sections
  Load: docs/stories/[STORY].md, docs/architecture/[SECTION].md

  Pitfall 2: "Merge conflicts in src/main.rs"

  Solution: Only one agent should touch main.rs at a time
  (PIPELINE-001)

  Pitfall 3: "Tests failing after integration"

  Solution: Run full test suite after each story: cargo test
  --all

  Pitfall 4: "Agent created wrong code structure"

  Solution: Verify agent loaded docs/architecture/source-tree.md

  Pitfall 5: "Lost track of what's complete"

  Solution: Update PROJECT-STATUS.md religiously after each
  story

  ---
  🎯 Success Criteria

  You know you're done with MVP when:

  - ✅ cargo build --release succeeds
  - ✅ cargo test --all passes
  - ✅ cargo clippy no warnings
  - ✅ Run ./target/release/crabmusic
  - ✅ Play music on your system
  - ✅ Sine wave appears and reacts to audio
  - ✅ Smooth 60 FPS rendering
  - ✅ No audio dropouts or glitches
  - ✅ Ctrl+C exits cleanly
  - ✅ IT FEELS GOOD! 🎵🎨

  ---
  📞 Quick Reference Commands

  Start Dev Agent:

  /BMad:agents:dev

  Load Required Context:

  Load these files:
  - docs/stories/[STORY-ID].md
  - docs/architecture/[RELEVANT-SECTION].md
  - docs/architecture/coding-standards.md

  Task: Implement [STORY-ID] - [Story Title]

  Follow acceptance criteria and testing requirements exactly.

  Check Status:

  # See what stories exist
  ls docs/stories/*.md

  # Check project compiles
  cargo build

  # Run tests
  cargo test

  # Lint
  cargo clippy

  Commit Progress:

  git add .
  git commit -m "Implement STORY-ID: description"
  git push

  ---
  🎉 Final Checklist Before Starting

  - Read this entire guide
  - Understand critical path dependencies
  - Have PROJECT-STATUS.md ready for tracking
  - Decide: Start with story completion OR jump to
  implementation?
  - Open 2-3 terminal windows for parallel agents
  - Play some music to test with! 🎵

  ---
  🚀 Ready to Launch?

  Recommended First Command:

  /BMad:agents:dev

  We're starting implementation of crabmusic - an ASCII music
  visualizer in Rust.

  Load:
  - docs/stories/FOUND-001-project-setup.md
  - docs/architecture/tech-stack.md
  - docs/architecture/source-tree.md
  - docs/architecture/coding-standards.md

  Task: Implement FOUND-001 - Project Setup and Scaffolding

  This creates the complete Rust project structure with all
  dependencies and module organization.

  Follow the acceptance criteria exactly. When done, verify:
  - cargo build compiles
  - cargo test runs
  - cargo clippy passes

  This is the foundation everything else builds on. Let's nail
  it!

  ---
  Good luck! You're going to build something awesome! 🦀🎵🎨

  ~ Winston the Architect 🏗️

  ---
  P.S. When you get that first sine wave reacting to music...
  that's going to be a MOMENT. Enjoy it! 🎉
