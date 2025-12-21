#!/bin/bash
# Watch act progress in real-time

echo "Monitoring act progress... (Press Ctrl+C to stop monitoring)"
echo ""

while true; do
    clear
    echo "=== Act Test Progress ==="
    echo ""
    
    # Show last 30 lines
    if [ -f /tmp/act-test-output.log ]; then
        tail -30 /tmp/act-test-output.log | sed 's/\[DEBUG\]//' | grep -E "\[CI/Test Suite\]|✅|❌|ERRO|WARN|Installing|Caching|Running|Check"
    else
        echo "Waiting for log file..."
    fi
    
    echo ""
    echo "=== Docker Images ==="
    docker images | grep "act\|catthehacker" | head -5
    
    sleep 3
done
