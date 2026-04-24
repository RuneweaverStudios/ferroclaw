#!/bin/bash
# Ferroclaw Test Execution Script

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results file
RESULTS_FILE="test_results_$(date +%Y%m%d_%H%M%S).txt"

echo "=== Ferroclaw Test Execution ==="
echo "Started: $(date)" | tee "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# Function to print section header
print_section() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}" | tee -a "$RESULTS_FILE"
    echo -e "${BLUE}$1${NC}" | tee -a "$RESULTS_FILE"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}" | tee -a "$RESULTS_FILE"
    echo "" | tee -a "$RESULTS_FILE"
}

# Function to run and capture test results
run_test() {
    local name="$1"
    local command="$2"

    echo -e "${YELLOW}Running: $name${NC}" | tee -a "$RESULTS_FILE"
    echo "Command: $command" | tee -a "$RESULTS_FILE"

    if eval "$command" >> "$RESULTS_FILE" 2>&1; then
        echo -e "${GREEN}✅ PASSED: $name${NC}" | tee -a "$RESULTS_FILE"
        return 0
    else
        echo -e "${RED}❌ FAILED: $name${NC}" | tee -a "$RESULTS_FILE"
        echo "Check $RESULTS_FILE for details" | tee -a "$RESULTS_FILE"
        return 1
    fi
}

# Phase 1: Library Tests
print_section "Phase 1: Library Tests"

run_test "Unit Tests (all modules)" "cargo test --lib --quiet" || true
run_test "TaskSystem Tests" "cargo test --lib tasks --quiet" || true
run_test "Security Tests" "cargo test --lib security --quiet" || true
run_test "MCP/DietMCP Tests" "cargo test --lib mcp --quiet" || true

# Phase 2: Integration Tests
print_section "Phase 2: Integration Tests"

run_test "All Integration Tests" "cargo test --tests --quiet" || true
run_test "Agent Integration" "cargo test --test integration_agent --quiet" || true
run_test "Security Integration" "cargo test --test integration_security --quiet" || true
run_test "Memory Integration" "cargo test --test integration_memory --quiet" || true
run_test "Config Integration" "cargo test --test integration_config --quiet" || true

# Phase 3: Feature-Specific Tests
print_section "Phase 3: Feature-Specific Tests"

run_test "Diet Compression Tests" "cargo test --test integration_diet --quiet" || true
run_test "Channels Tests" "cargo test --test integration_channels --quiet" || true
run_test "Providers Tests" "cargo test --test integration_providers --quiet" || true
run_test "Skills Tests" "cargo test --test integration_skills --quiet" || true
run_test "WebSocket Tests" "cargo test --test integration_websocket --quiet" || true

# Phase 4: Benchmarks
print_section "Phase 4: Performance Benchmarks"

run_test "Diet Compression Benchmark" "cargo bench --bench diet_compression --quiet" || true
run_test "Memory Store Benchmark" "cargo bench --bench memory_store --quiet" || true
run_test "Security Audit Benchmark" "cargo bench --bench security_audit --quiet" || true

# Summary
print_section "Test Summary"

echo "Test execution completed at: $(date)" | tee -a "$RESULTS_FILE"
echo "Full results saved to: $RESULTS_FILE" | tee -a "$RESULTS_FILE"
echo ""

# Quick summary extraction
echo -e "${GREEN}Quick Test Summary:${NC}"
grep -E "(✅|❌)" "$RESULTS_FILE" | tail -10 || echo "Test results parsing failed"

echo ""
echo "To view full results: cat $RESULTS_FILE"
echo ""
