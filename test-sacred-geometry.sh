#!/usr/bin/env bash
# Sacred Geometry Visual Test Script
# Runs all three sacred geometry demos for human visual testing
# Unix/macOS compatible version

# Colors
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
GREEN='\033[0;32m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

echo -e "${CYAN}🦀 CrabMusic - Sacred Geometry Visual Test Suite${NC}"
echo -e "${CYAN}=================================================${NC}"
echo ""

# Function to run a demo
run_demo() {
    local name="$1"
    local example="$2"
    local description="$3"
    local controls="$4"

    echo -e "${YELLOW}🌟 $name${NC}"
    echo -e "${GRAY}   $description${NC}"
    echo -e "${GRAY}   Controls: $controls${NC}"
    echo ""
    echo -e "${GREEN}   Press any key to start...${NC}"
    read -n 1 -s

    echo ""
    cargo run --example "$example"

    echo ""
    echo -e "${GREEN}   Demo complete!${NC}"
    echo ""
}

# Test 1: Anti-Aliasing Demo
echo -e "${MAGENTA}📋 Test 1: Anti-Aliasing Comparison${NC}"
echo -e "${GRAY}   This demo shows the difference between binary and anti-aliased rendering.${NC}"
echo ""
echo -e "${GREEN}   Press any key to start...${NC}"
read -n 1 -s
echo ""
cargo run --example aa_demo
echo ""
echo -e "${GREEN}   ✅ Anti-aliasing demo complete!${NC}"
echo ""
echo -e "${YELLOW}   Press any key to continue to Flower of Life...${NC}"
read -n 1 -s
echo ""

# Test 2: Flower of Life
run_demo \
    "🌸 Flower of Life Visualizer" \
    "flower_of_life_demo" \
    "Hexagonal overlapping circles with audio-reactive animation" \
    "q=quit, c=cycle colors, +/-=adjust rings"

echo -e "${YELLOW}   Press any key to continue to Mandala...${NC}"
read -n 1 -s
echo ""

# Test 3: Mandala Generator
run_demo \
    "🕉️  Mandala Generator" \
    "mandala_demo" \
    "Radial symmetry with layered patterns and independent rotation" \
    "q=quit, c=cycle colors, s=change symmetry, +/-=adjust layers"

echo -e "${YELLOW}   Press any key to continue to combined demo...${NC}"
read -n 1 -s
echo ""

# Test 4: Combined Demo
run_demo \
    "✨ Sacred Geometry Combined Demo" \
    "sacred_geometry_demo" \
    "Toggle between Flower of Life and Mandala" \
    "q=quit, v=switch visualizer, c=cycle colors"

# Summary
echo ""
echo -e "${CYAN}=================================================${NC}"
echo -e "${GREEN}🎉 All Visual Tests Complete!${NC}"
echo -e "${CYAN}=================================================${NC}"
echo ""
echo -e "${YELLOW}Visual Quality Checklist:${NC}"
echo -e "${GRAY}  [ ] Circles are smooth (no jagged edges)${NC}"
echo -e "${GRAY}  [ ] Lines are smooth${NC}"
echo -e "${GRAY}  [ ] Rotation is smooth${NC}"
echo -e "${GRAY}  [ ] Pulse effect is visible${NC}"
echo -e "${GRAY}  [ ] Beat flash is noticeable${NC}"
echo -e "${GRAY}  [ ] Colors cycle smoothly${NC}"
echo -e "${GRAY}  [ ] Patterns maintain symmetry${NC}"
echo -e "${GRAY}  [ ] 60 FPS performance${NC}"
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo -e "${GRAY}  1. Review visual quality${NC}"
echo -e "${GRAY}  2. Test with different terminal sizes${NC}"
echo -e "${GRAY}  3. Integrate with main CrabMusic app${NC}"
echo -e "${GRAY}  4. Test with real audio input${NC}"
echo ""
echo -e "${CYAN}✨ Ready for production! 🦀${NC}"
