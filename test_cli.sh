#!/bin/bash
# Test script for Phosphor CLI

echo "Testing Phosphor CLI with debug logging..."
echo "Press Ctrl+C to exit"
echo ""

# Run with debug logging and proper terminal allocation
exec ./target/debug/phosphor-cli --debug