#!/bin/bash
# This script runs the integration tests for the Meta Programming Language tool

echo "Running integration tests for Meta Programming Language tool..."
echo ""

# Run the tests with detailed output
RUST_BACKTRACE=1 cargo test --test integration_tests -- --nocapture

echo ""
echo "Tests completed. Check the output above for any failures."
