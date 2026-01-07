#!/bin/bash

echo "=== Testing Terminal Context Awareness ==="
echo ""

echo "Error: File not found"
echo "FATAL: Connection refused"
echo "Warning: Deprecated API usage"
echo "WARN: Low disk space"
echo ""

echo "=== Stack Traces ==="
echo "thread 'main' panicked at src/main.rs:42:10:"
echo "  File \"/home/user/project/test.py\", line 123, in main"
echo "  at /home/user/app/src/lib.rs:156:9"
echo "  /usr/local/project/handler.go:89:12"
echo ""

echo "=== Mixed Content ==="
echo "INFO: Starting server on port 8080"
echo "Error: Could not bind to src/server.rs:25:5"
echo "Success: Operation completed"
echo ""

echo "Click on any file path to open it in VS Code!"
