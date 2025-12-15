#!/bin/bash
# Test script to demonstrate the interactive execution prompt

echo "Testing cmdai with interactive execution prompt..."
echo ""
echo "Running: cmdai 'list files'"
echo ""
echo "NOTE: When you run this command, you'll see:"
echo "  1. The generated command"
echo "  2. An explanation"
echo "  3. A prompt: 'Execute this command? (y/N)'"
echo ""
echo "Press 'y' to execute, or 'n' (or Enter) to skip."
echo ""
echo "---"
echo ""

cargo run --quiet -- "list files"
