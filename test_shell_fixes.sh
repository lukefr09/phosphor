#!/bin/bash
# Test script for debugging shell output closure

echo "Building phosphor-cli..."
cargo build --bin phosphor-cli

echo -e "\n=== Test 1: bash with --noprofile --norc ==="
echo "Should bypass shell config files"
echo "Press Ctrl+C to continue"
./target/debug/phosphor-cli --debug --shell /bin/bash

echo -e "\n=== Test 2: sh with minimal environment ==="
echo "Using env -i for clean environment"
echo "Press Ctrl+C to continue"
./target/debug/phosphor-cli --debug --shell /bin/sh --minimal-env

echo -e "\n=== Test 3: zsh with --no-rcs ==="
echo "Should skip all zsh rc files"
echo "Press Ctrl+C to continue"
./target/debug/phosphor-cli --debug --shell /bin/zsh

echo -e "\n=== Test 4: bash with minimal environment ==="
echo "Cleanest possible environment"
echo "Press Ctrl+C to exit"
./target/debug/phosphor-cli --debug --shell /bin/bash --minimal-env