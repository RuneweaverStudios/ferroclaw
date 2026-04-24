#!/bin/bash
# Ferroclaw Test Environment Setup Script

set -e

echo "=== Ferroclaw Test Environment Setup ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check required tools
echo -e "${YELLOW}Checking required tools...${NC}"

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found"
    exit 1
fi
echo "✅ Cargo found: $(cargo --version)"

if ! command -v git &> /dev/null; then
    echo "❌ Git not found"
    exit 1
fi
echo "✅ Git found: $(git --version)"

if ! command -v npx &> /dev/null; then
    echo "⚠️  npx not found (required for MCP filesystem server)"
fi

# Environment variables
echo ""
echo -e "${YELLOW}Setting up environment variables...${NC}"

# Test configuration
export FERROCLAW_TEST_MODE="true"
export FERROCLAW_CONFIG="$(pwd)/ferroclaw_test.toml"
export FERROCLAW_LOG_LEVEL="debug"

echo "✅ FERROCLAW_TEST_MODE=$FERROCLAW_TEST_MODE"
echo "✅ FERROCLAW_CONFIG=$FERROCLAW_CONFIG"
echo "✅ FERROCLAW_LOG_LEVEL=$FERROCLAW_LOG_LEVEL"

# Create test directories
echo ""
echo -e "${YELLOW}Creating test directories...${NC}"

mkdir -p test_data/tasks
mkdir -p test_data/memdir
mkdir -p test_data/audit
mkdir -p test_output

echo "✅ Test directories created"

# Verify project structure
echo ""
echo -e "${YELLOW}Verifying project structure...${NC}"

required_dirs=("src" "tests" "benches" "docs")
for dir in "${required_dirs[@]}"; do
    if [ -d "$dir" ]; then
        echo "✅ $dir/ exists"
    else
        echo "❌ $dir/ not found"
        exit 1
    fi
done

# Check for critical source files
required_files=("src/lib.rs" "src/main.rs" "src/cli.rs" "Cargo.toml")
for file in "${required_files[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file exists"
    else
        echo "❌ $file not found"
        exit 1
    fi
done

# Verify test files
echo ""
echo -e "${YELLOW}Verifying test files...${NC}"

test_count=$(find tests -name "*.rs" | wc -l | tr -d ' ')
echo "✅ Found $test_count integration test files"

bench_count=$(find benches -name "*.rs" | wc -l | tr -d ' ')
echo "✅ Found $bench_count benchmark files"

# Display test configuration
echo ""
echo -e "${YELLOW}Test Configuration Summary:${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test Mode:           $FERROCLAW_TEST_MODE"
echo "Config File:         $FERROCLAW_CONFIG"
echo "Log Level:           $FERROCLAW_LOG_LEVEL"
echo "Integration Tests:    $test_count files"
echo "Benchmarks:          $bench_count files"
echo "Test Data Directory:  $(pwd)/test_data"
echo "Output Directory:    $(pwd)/test_output"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Summary
echo ""
echo -e "${GREEN}=== Test Environment Setup Complete ===${NC}"
echo ""
echo "Next steps:"
echo "  1. Run: cargo test --lib           # Library tests"
echo "  2. Run: cargo test --tests         # Integration tests"
echo "  3. Run: cargo bench                 # Performance benchmarks"
echo ""
echo "To set environment variables manually:"
echo "  export FERROCLAW_TEST_MODE=true"
echo "  export FERROCLAW_CONFIG=$(pwd)/ferroclaw_test.toml"
echo ""
