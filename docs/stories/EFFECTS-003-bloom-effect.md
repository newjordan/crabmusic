# EFFECTS-003: Bloom Effect

**Status**: In Progress  
**Priority**: P1 (High)  
**Estimated Effort**: 4-6 hours  
**Dependencies**: EFFECTS-007 (Effect Pipeline Framework) ✅  
**Blocks**: None

---

## 📋 Story Description

Implement a bloom/glow effect that makes bright elements "glow" by blurring and adding luminosity to pixels above a brightness threshold. This creates a dreamy, ethereal aesthetic commonly seen in modern games and visualizers.

The bloom effect works by:
1. Identifying bright pixels (above threshold)
2. Extracting them to a separate layer
3. Applying Gaussian blur to the bright layer
4. Compositing the blurred layer back onto the original

This is a **moderately complex effect** that requires implementing blur algorithms and multi-pass rendering.

---

## 🎯 Acceptance Criteria

### Functional Requirements
- [ ] `BloomEffect` struct implements the `Effect` trait
- [ ] Configurable brightness threshold (0.0-1.0, pixels above threshold bloom)
- [ ] Configurable bloom intensity (0.0-1.0, strength of glow)
- [ ] Configurable blur radius (1-5, size of glow)
- [ ] Bloom only affects bright pixels (preserves dark areas)
- [ ] Additive blending (bloom adds to original, doesn't replace)
- [ ] Effect can be toggled on/off via the Effect trait interface
- [ ] Effect respects the `enabled` flag

### Visual Requirements
- [ ] Bright elements have visible glow/halo
- [ ] Glow extends beyond original bright pixels
- [ ] Smooth gradient falloff (no hard edges)
- [ ] Effect works with all color schemes
- [ ] Effect works with all visualizers
- [ ] No color banding or artifacts
- [ ] Glow respects color of original bright pixels

### Performance Requirements
- [ ] Bloom effect completes in <5ms for 200x100 grid
- [ ] Blur algorithm is optimized (separable Gaussian)
- [ ] Minimal memory allocations during apply()

### Testing Requirements
- [ ] Unit tests for BloomEffect creation
- [ ] Unit tests for enable/disable functionality
- [ ] Unit tests for threshold/intensity/radius configuration
- [ ] Unit tests for brightness extraction
- [ ] Unit tests for blur algorithm
- [ ] Performance benchmark test
- [ ] Visual validation with all visualizers

---

## 🏗️ Technical Approach

### Implementation Strategy

**File**: `src/effects/bloom.rs`

```rust
pub struct BloomEffect {
    enabled: bool,
    intensity: f32,           // 0.0-1.0, strength of bloom
    threshold: f32,           // 0.0-1.0, brightness threshold for bloom
    blur_radius: usize,       // 1-5, size of blur kernel
    // Temporary buffers for multi-pass rendering
    bright_buffer: Vec<Option<Color>>,
    blur_buffer: Vec<Option<Color>>,
}

impl BloomEffect {
    pub fn new(threshold: f32, blur_radius: usize) -> Self;
    pub fn threshold(&self) -> f32;
    pub fn set_threshold(&mut self, threshold: f32);
    pub fn blur_radius(&self) -> usize;
    pub fn set_blur_radius(&mut self, radius: usize);
}

impl Effect for BloomEffect {
    fn apply(&mut self, grid: &mut GridBuffer, _params: &AudioParameters) {
        if !self.enabled { return; }
        
        let width = grid.width();
        let height = grid.height();
        
        // Resize buffers if needed
        self.resize_buffers(width, height);
        
        // Pass 1: Extract bright pixels above threshold
        self.extract_bright_pixels(grid);
        
        // Pass 2: Apply separable Gaussian blur (horizontal)
        self.blur_horizontal(width, height);
        
        // Pass 3: Apply separable Gaussian blur (vertical)
        self.blur_vertical(width, height);
        
        // Pass 4: Composite blurred bloom back onto original (additive)
        self.composite_bloom(grid);
    }
    
    // ... trait methods
}
```

### Algorithm Details

#### 1. Brightness Extraction
```rust
fn extract_bright_pixels(&mut self, grid: &GridBuffer) {
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let cell = grid.get_cell(x, y);
            if let Some(color) = cell.foreground_color {
                // Calculate perceived brightness (luminance)
                let brightness = color_brightness(color);
                
                if brightness >= self.threshold {
                    // Store bright pixel
                    self.bright_buffer[y * grid.width() + x] = Some(color);
                } else {
                    self.bright_buffer[y * grid.width() + x] = None;
                }
            }
        }
    }
}

fn color_brightness(color: Color) -> f32 {
    // Perceived luminance formula (ITU-R BT.709)
    (0.2126 * color.r as f32 + 0.7152 * color.g as f32 + 0.0722 * color.b as f32) / 255.0
}
```

#### 2. Separable Gaussian Blur
Use separable Gaussian blur for performance (O(n*r) instead of O(n*r²)):
- Horizontal pass: blur each row
- Vertical pass: blur each column

```rust
fn blur_horizontal(&mut self, width: usize, height: usize) {
    let kernel = gaussian_kernel(self.blur_radius);
    
    for y in 0..height {
        for x in 0..width {
            let mut r_sum = 0.0;
            let mut g_sum = 0.0;
            let mut b_sum = 0.0;
            let mut weight_sum = 0.0;
            
            for (i, &weight) in kernel.iter().enumerate() {
                let offset = i as isize - self.blur_radius as isize;
                let sample_x = (x as isize + offset).clamp(0, width as isize - 1) as usize;
                
                if let Some(color) = self.bright_buffer[y * width + sample_x] {
                    r_sum += color.r as f32 * weight;
                    g_sum += color.g as f32 * weight;
                    b_sum += color.b as f32 * weight;
                    weight_sum += weight;
                }
            }
            
            if weight_sum > 0.0 {
                self.blur_buffer[y * width + x] = Some(Color::new(
                    (r_sum / weight_sum) as u8,
                    (g_sum / weight_sum) as u8,
                    (b_sum / weight_sum) as u8,
                ));
            }
        }
    }
}
```

#### 3. Additive Compositing
```rust
fn composite_bloom(&self, grid: &mut GridBuffer) {
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            if let Some(bloom_color) = self.blur_buffer[y * grid.width() + x] {
                let cell = grid.get_cell_mut(x, y);
                if let Some(original_color) = cell.foreground_color {
                    // Additive blend with intensity
                    cell.foreground_color = Some(Color::new(
                        (original_color.r as f32 + bloom_color.r as f32 * self.intensity).min(255.0) as u8,
                        (original_color.g as f32 + bloom_color.g as f32 * self.intensity).min(255.0) as u8,
                        (original_color.b as f32 + bloom_color.b as f32 * self.intensity).min(255.0) as u8,
                    ));
                }
            }
        }
    }
}
```

### Integration Points

**Add to main.rs initialization**:
```rust
// Add bloom effect to pipeline
effect_pipeline.add_effect(Box::new(effects::bloom::BloomEffect::new(0.7, 2)));
```

**Export in src/effects/mod.rs**:
```rust
pub mod bloom;
```

---

## 🧪 Testing Strategy

### Unit Tests
- `test_bloom_new()` - Verify default initialization
- `test_bloom_enable_disable()` - Toggle functionality
- `test_bloom_threshold()` - Threshold clamping and application
- `test_bloom_intensity()` - Intensity clamping
- `test_bloom_blur_radius()` - Radius configuration
- `test_color_brightness()` - Brightness calculation
- `test_gaussian_kernel()` - Kernel generation
- `test_bloom_extract_bright()` - Bright pixel extraction
- `test_bloom_disabled()` - Verify disabled effect does nothing
- `test_bloom_performance()` - Benchmark <5ms for 200x100 grid

### Integration Tests
- Test with sine wave visualizer (bright peaks should glow)
- Test with spectrum visualizer (bright bars should glow)
- Test with oscilloscope visualizer (bright waveform should glow)
- Test with different thresholds (0.5, 0.7, 0.9)
- Test with different blur radii (1, 2, 3, 5)
- Test with different intensities (0.3, 0.5, 1.0)

---

## 📊 Success Metrics

- ✅ All unit tests pass
- ✅ Performance benchmark <5ms
- ✅ Visual validation with all visualizers
- ✅ No regressions in existing functionality
- ✅ Code review approved
- ✅ Documentation complete

---

## 🚀 Implementation Notes

### Design Decisions

1. **Separable Gaussian blur**: Use 1D horizontal + vertical passes instead of 2D kernel for O(n*r) performance
2. **Brightness threshold**: Use ITU-R BT.709 luminance formula for perceptually accurate brightness
3. **Additive blending**: Add bloom to original (don't replace) for authentic glow effect
4. **Clamped addition**: Clamp RGB to 255 to prevent overflow
5. **Temporary buffers**: Reuse buffers across frames to minimize allocations

### Gaussian Kernel Generation

```rust
fn gaussian_kernel(radius: usize) -> Vec<f32> {
    let sigma = radius as f32 / 2.0;
    let size = radius * 2 + 1;
    let mut kernel = vec![0.0; size];
    let mut sum = 0.0;
    
    for i in 0..size {
        let x = i as f32 - radius as f32;
        kernel[i] = (-x * x / (2.0 * sigma * sigma)).exp();
        sum += kernel[i];
    }
    
    // Normalize
    for i in 0..size {
        kernel[i] /= sum;
    }
    
    kernel
}
```

### Future Enhancements

- [ ] Audio-reactive bloom intensity (pulse with beat)
- [ ] Configurable bloom color (not just original color)
- [ ] Multiple blur passes for stronger glow
- [ ] Lens flare effect (star-shaped bloom)
- [ ] HDR tone mapping

### Known Limitations

- Only works with colored output (no effect on monochrome)
- Blur radius limited to 5 for performance
- No sub-pixel positioning (terminal grid aligned)
- Temporary buffers increase memory usage

---

## 📝 Checklist

- [ ] Create `src/effects/bloom.rs`
- [ ] Implement `BloomEffect` struct
- [ ] Implement brightness extraction
- [ ] Implement Gaussian kernel generation
- [ ] Implement separable blur (horizontal + vertical)
- [ ] Implement additive compositing
- [ ] Implement `Effect` trait for `BloomEffect`
- [ ] Export module in `src/effects/mod.rs`
- [ ] Add to effect pipeline in `src/main.rs`
- [ ] Write unit tests
- [ ] Write performance benchmark
- [ ] Test with all visualizers
- [ ] Update documentation
- [ ] Code review
- [ ] Commit and push

---

## 🔗 Related Stories

- **EFFECTS-007**: Effect Pipeline Framework (dependency) ✅
- **EFFECTS-004**: Scanline Effect (completed) ✅
- **EFFECTS-005**: CRT Curve & Distortion (complementary effect)
- **EFFECTS-006**: Phosphor Glow Effect (similar temporal effect)

---

**Story Created**: 2025-01-03  
**Last Updated**: 2025-01-03

