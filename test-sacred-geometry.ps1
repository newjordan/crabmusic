# Sacred Geometry Visual Test Script
# Runs all three sacred geometry demos for human visual testing

Write-Host "🦀 CrabMusic - Sacred Geometry Visual Test Suite" -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host ""

# Function to run a demo
function Run-Demo {
    param(
        [string]$Name,
        [string]$Example,
        [string]$Description,
        [string]$Controls
    )
    
    Write-Host "🌟 $Name" -ForegroundColor Yellow
    Write-Host "   $Description" -ForegroundColor Gray
    Write-Host "   Controls: $Controls" -ForegroundColor Gray
    Write-Host ""
    Write-Host "   Press any key to start..." -ForegroundColor Green
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    
    Write-Host ""
    cargo run --example $Example
    
    Write-Host ""
    Write-Host "   Demo complete!" -ForegroundColor Green
    Write-Host ""
}

# Test 1: Anti-Aliasing Demo
Write-Host "📋 Test 1: Anti-Aliasing Comparison" -ForegroundColor Magenta
Write-Host "   This demo shows the difference between binary and anti-aliased rendering." -ForegroundColor Gray
Write-Host ""
Write-Host "   Press any key to start..." -ForegroundColor Green
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
Write-Host ""
cargo run --example aa_demo
Write-Host ""
Write-Host "   ✅ Anti-aliasing demo complete!" -ForegroundColor Green
Write-Host ""
Write-Host "   Press any key to continue to Flower of Life..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
Write-Host ""

# Test 2: Flower of Life
Run-Demo `
    -Name "🌸 Flower of Life Visualizer" `
    -Example "flower_of_life_demo" `
    -Description "Hexagonal overlapping circles with audio-reactive animation" `
    -Controls "q=quit, c=cycle colors, +/-=adjust rings"

Write-Host "   Press any key to continue to Mandala..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
Write-Host ""

# Test 3: Mandala Generator
Run-Demo `
    -Name "🕉️  Mandala Generator" `
    -Example "mandala_demo" `
    -Description "Radial symmetry with layered patterns and independent rotation" `
    -Controls "q=quit, c=cycle colors, s=change symmetry, +/-=adjust layers"

Write-Host "   Press any key to continue to combined demo..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
Write-Host ""

# Test 4: Combined Demo
Run-Demo `
    -Name "✨ Sacred Geometry Combined Demo" `
    -Example "sacred_geometry_demo" `
    -Description "Toggle between Flower of Life and Mandala" `
    -Controls "q=quit, v=switch visualizer, c=cycle colors"

# Summary
Write-Host ""
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host "🎉 All Visual Tests Complete!" -ForegroundColor Green
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Visual Quality Checklist:" -ForegroundColor Yellow
Write-Host "  [ ] Circles are smooth (no jagged edges)" -ForegroundColor Gray
Write-Host "  [ ] Lines are smooth" -ForegroundColor Gray
Write-Host "  [ ] Rotation is smooth" -ForegroundColor Gray
Write-Host "  [ ] Pulse effect is visible" -ForegroundColor Gray
Write-Host "  [ ] Beat flash is noticeable" -ForegroundColor Gray
Write-Host "  [ ] Colors cycle smoothly" -ForegroundColor Gray
Write-Host "  [ ] Patterns maintain symmetry" -ForegroundColor Gray
Write-Host "  [ ] 60 FPS performance" -ForegroundColor Gray
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Yellow
Write-Host "  1. Review visual quality" -ForegroundColor Gray
Write-Host "  2. Test with different terminal sizes" -ForegroundColor Gray
Write-Host "  3. Integrate with main CrabMusic app" -ForegroundColor Gray
Write-Host "  4. Test with real audio input" -ForegroundColor Gray
Write-Host ""
Write-Host "✨ Ready for production! 🦀" -ForegroundColor Cyan

