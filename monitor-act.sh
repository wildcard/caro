#!/bin/bash
# Real-time act monitoring

echo "Monitoring act CI test run..."
echo "Press Ctrl+C to stop monitoring (test continues in background)"
echo ""

while true; do
    clear
    echo "═══════════════════════════════════════════════════════════════"
    echo "                    act CI Test Progress                       "
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    
    # Show current step
    echo "Current Step:"
    tail -100 /tmp/act-full-test.log 2>/dev/null | \
        grep -E "\[CI/Test Suite.*\] ⭐ Run" | tail -3
    echo ""
    
    # Show recent completions
    echo "Recent Completions:"
    tail -100 /tmp/act-full-test.log 2>/dev/null | \
        grep -E "✅|❌" | tail -5
    echo ""
    
    # Show any errors
    echo "Recent Errors:"
    tail -100 /tmp/act-full-test.log 2>/dev/null | \
        grep -E "Error|ERRO|Failed|❌" | tail -5 || echo "  (none)"
    echo ""
    
    # Show test output
    echo "Recent Test Activity:"
    tail -50 /tmp/act-full-test.log 2>/dev/null | \
        grep -E "Running|Compiling|Finished|test " | tail -5 || echo "  (waiting...)"
    echo ""
    
    echo "═══════════════════════════════════════════════════════════════"
    echo "Full log: /tmp/act-full-test.log"
    echo "═══════════════════════════════════════════════════════════════"
    
    sleep 3
done
