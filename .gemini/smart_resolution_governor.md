# Smart Resolution Governor 🎯

## Overview
Intelligent camera resolution selection system that automatically chooses the optimal camera format based on your terminal size for maximum performance.

## How It Works

### 1. **Terminal Analysis**
```
Terminal: 200x50 cells = 400x200 pixels (Braille dots)
```
- Measures your terminal dimensions in cells
- Calculates actual pixel resolution (2x4 Braille dots per cell)

### 2. **Optimal Resolution Calculation**
```
Optimal = Terminal Resolution × 2
```
- Uses 2x scaling factor for quality
- Example: 400x200 terminal → 800x400 optimal camera resolution
- Beyond 2x provides diminishing returns

### 3. **Smart Scoring Algorithm**

Each available camera format is scored based on:

#### **Size Penalty** (avoid wasted processing)
- If resolution > 1.5x optimal: Heavy penalty
- Processing 1080p for a 400x200 terminal wastes 90% of pixels!

#### **Quality Penalty** (avoid pixelation)
- If resolution < 0.8x optimal: Quality penalty
- Too low resolution causes visible artifacts

#### **Common Resolution Bonus**
- 640x480, 320x240, 800x600: -20 points (well-optimized)
- 1280x720, 1920x1080: Penalty if too large for terminal

#### **Total Pixel Penalty**
- Slight penalty for total megapixels
- Encourages efficiency

### 4. **Selection**
The format with the **lowest score wins** and is automatically selected.

## Example Scenarios

### Small Terminal (80x24 cells = 160x96 pixels)
- **Optimal**: 320x192
- **Likely Selection**: 320x240 (common format, close match)
- **FPS**: ~40-60 FPS

### Medium Terminal (200x50 cells = 400x200 pixels)
- **Optimal**: 800x400
- **Likely Selection**: 640x480 or 800x600
- **FPS**: ~25-35 FPS

### Large Terminal (300x80 cells = 600x320 pixels)
- **Optimal**: 1200x640
- **Likely Selection**: 1280x720
- **FPS**: ~15-25 FPS

### Full Screen (400x100 cells = 800x400 pixels)
- **Optimal**: 1600x800
- **Likely Selection**: 1280x720 or 1920x1080
- **FPS**: ~10-20 FPS

## Performance Monitoring

### Real-Time FPS Display
```
WEBCAM [28.3 FPS] | +/- thr=128 | a auto=OFF | c color=OFF
```

The HUD now shows:
- **FPS**: Rolling 30-frame average for smooth readings
- **Threshold**: Current threshold value
- **Auto**: Auto-threshold status
- **Color Mode**: Current color processing mode

### Logging Output
When you start the webcam, you'll see:
```
INFO Terminal: 200x50 cells = 400x200 pixels (Braille dots)
INFO Available camera formats: 12
DEBUG Format 1920x1080: scale=2.63x, score=163.0 (size_pen=113.0, qual_pen=0.0)
DEBUG Format 1280x720: scale=1.69x, score=31.8 (size_pen=19.0, qual_pen=0.0)
DEBUG Format 640x480: scale=0.88x, score=4.6 (size_pen=0.0, qual_pen=0.0)
INFO 🎯 Smart Governor selected: 640x480 (score: 4.6, optimal was 800x400)
```

## Performance Tips

### For Maximum Speed:
1. **Smaller terminal window** = Higher FPS
2. **ColorMode::Off** = Fastest (no color processing)
3. **Manual threshold** = Faster than auto (Otsu algorithm)
4. **Disable HUD** (F1) = Tiny boost

### For Best Quality:
1. **Larger terminal window** = More detail
2. **ColorMode::Full** = Full RGB color
3. **Auto threshold** = Better edge detection
4. **Enable HUD** = Monitor performance

## Governor Benefits

✅ **Automatic**: No manual configuration needed  
✅ **Adaptive**: Adjusts to terminal resizes  
✅ **Efficient**: Minimizes wasted processing  
✅ **Smart**: Balances quality vs performance  
✅ **Transparent**: Shows you what it selected and why  

## Technical Details

### Why 2x Scaling?
- 1x: Aliasing artifacts, poor quality
- 2x: Sweet spot - good quality, minimal overhead
- 4x+: Diminishing returns, wasted processing

### Why Nearest Neighbor Filtering?
- 3-5x faster than Triangle/Lanczos
- Quality difference imperceptible at terminal resolution
- Terminal rendering is already "pixelated" by nature

### Why Rolling Average FPS?
- Smooths out frame time variance
- More readable than instant FPS
- 30-frame window balances responsiveness vs stability
