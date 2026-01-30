#!/bin/bash
# Benchmark script to compare different optimization levels
# Run this to see the impact of various optimizations

set -e

echo "==================================================================="
echo "Binius DAS Performance Benchmark - Optimization Level Comparison"
echo "==================================================================="
echo ""
echo "This script will run the same workload with different optimization"
echo "levels to demonstrate the performance differences."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to extract timing from output
extract_timing() {
    grep "Evaluation claim computed in" | sed -n 's/.*computed in \([0-9]*\) ms.*/\1/p'
}

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean -q
echo ""

# 1. Debug build (opt-level = 0)
echo "================================================"
echo "1. DEBUG BUILD (opt-level = 0)"
echo "================================================"
echo "Compiling..."
cargo build -q 2>&1 | grep -v "Compiling" || true
echo "Running..."
echo -e "${YELLOW}Expect: ~12,000 ms for evaluation claim${NC}"
DEBUG_TIME=$(cargo run -q 2>&1 | extract_timing | tail -1)
echo -e "Result: ${RED}${DEBUG_TIME} ms${NC}"
echo ""

# 2. Release build (opt-level = 3)
echo "================================================"
echo "2. RELEASE BUILD (opt-level = 3, default)"
echo "================================================"
echo "Compiling..."
cargo build --release -q 2>&1 | grep -v "Compiling" || true
echo "Running..."
echo -e "${YELLOW}Expect: ~900 ms for evaluation claim${NC}"
RELEASE_TIME=$(cargo run --release -q 2>&1 | extract_timing | tail -1)
echo -e "Result: ${GREEN}${RELEASE_TIME} ms${NC}"
echo ""

# 3. Dev-optimized build (opt-level = 2 for deps, 0 for your code)
echo "================================================"
echo "3. DEV-OPTIMIZED BUILD (opt-level = 2 for deps)"
echo "================================================"
echo "This profile optimizes dependencies but keeps your code"
echo "unoptimized for faster iteration during development."
echo ""
echo "Compiling..."
cargo build --profile=dev-optimized -q 2>&1 | grep -v "Compiling" || true
echo "Running..."
echo -e "${YELLOW}Expect: ~2,000-4,000 ms (between debug and release)${NC}"
DEV_OPT_TIME=$(cargo run --profile=dev-optimized -q 2>&1 | extract_timing | tail -1)
echo -e "Result: ${YELLOW}${DEV_OPT_TIME} ms${NC}"
echo ""

# 4. Release with parallel feature
echo "================================================"
echo "4. RELEASE + PARALLEL FEATURE"
echo "================================================"
echo "Uses rayon for multi-core processing."
echo ""
echo "Compiling..."
cargo build --release --features parallel -q 2>&1 | grep -v "Compiling" || true
echo "Running..."
echo -e "${YELLOW}Expect: Similar to release (parallel not used in main hot paths)${NC}"
PARALLEL_TIME=$(cargo run --release --features parallel -q 2>&1 | extract_timing | tail -1)
echo -e "Result: ${GREEN}${PARALLEL_TIME} ms${NC}"
echo ""

# Summary
echo "==================================================================="
echo "                        SUMMARY"
echo "==================================================================="
echo ""
printf "%-30s %10s %10s\n" "Build Configuration" "Time (ms)" "Speedup"
echo "-------------------------------------------------------------------"
printf "%-30s %10s %10s\n" "Debug (opt-level=0)" "$DEBUG_TIME" "1.0x"

if [ -n "$DEBUG_TIME" ] && [ -n "$RELEASE_TIME" ]; then
    RELEASE_SPEEDUP=$(echo "scale=1; $DEBUG_TIME / $RELEASE_TIME" | bc)
    printf "%-30s %10s %10s\n" "Release (opt-level=3)" "$RELEASE_TIME" "${RELEASE_SPEEDUP}x"
fi

if [ -n "$DEBUG_TIME" ] && [ -n "$DEV_OPT_TIME" ]; then
    DEV_SPEEDUP=$(echo "scale=1; $DEBUG_TIME / $DEV_OPT_TIME" | bc)
    printf "%-30s %10s %10s\n" "Dev-optimized" "$DEV_OPT_TIME" "${DEV_SPEEDUP}x"
fi

if [ -n "$DEBUG_TIME" ] && [ -n "$PARALLEL_TIME" ]; then
    PARALLEL_SPEEDUP=$(echo "scale=1; $DEBUG_TIME / $PARALLEL_TIME" | bc)
    printf "%-30s %10s %10s\n" "Release + Parallel" "$PARALLEL_TIME" "${PARALLEL_SPEEDUP}x"
fi

echo "==================================================================="
echo ""
echo "✅ Benchmark complete!"
echo ""
echo "Key Takeaways:"
echo "1. Release build is ~14-15x faster than debug (expected)"
echo "2. Use 'dev-optimized' profile for faster iteration during development"
echo "3. Always use '--release' for benchmarking and production"
echo "4. The 'parallel' feature helps with specific operations (see code)"
echo ""
echo "For more details, see: OPTIMIZATION_ANALYSIS.md"




