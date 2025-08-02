#!/bin/bash
# Debug script to test different shells

echo "Building phosphor-cli..."
cargo build --bin phosphor-cli

echo -e "\n=== Testing with /bin/sh ==="
echo "Press Ctrl+C to continue to next test"
./target/debug/phosphor-cli --debug --shell /bin/sh

echo -e "\n=== Testing with /bin/bash ==="
echo "Press Ctrl+C to continue to next test"
./target/debug/phosphor-cli --debug --shell /bin/bash

echo -e "\n=== Testing with default shell (zsh) ==="
echo "Press Ctrl+C to exit"
./target/debug/phosphor-cli --debug