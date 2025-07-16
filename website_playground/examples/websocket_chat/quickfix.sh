#!/bin/bash

echo "🔧 DFIR WebSocket Chat - Quick Fix & Test"
echo "========================================="

# Check if HTTP server is running
echo "🔍 Checking HTTP server..."
if curl -s -o /dev/null -w "%{http_code}" http://localhost:8081/examples/websocket_chat/ | grep -q "200"; then
    echo "✅ HTTP server is running on port 8081"
    echo "🌐 Web app accessible at: http://localhost:8081/examples/websocket_chat/"
else
    echo "❌ HTTP server not accessible. Starting new server..."
    cd /Users/jmh/code/hydro/website_playground
    python3 -m http.server 8081 &
    sleep 2
    echo "✅ HTTP server started on port 8081"
fi

# Check if WebSocket server is running
echo "🔍 Checking WebSocket server..."
if nc -z 127.0.0.1 3000 2>/dev/null; then
    echo "✅ WebSocket server is running on port 3000"
else
    echo "❌ WebSocket server not running. Please start it manually:"
    echo "   cd /Users/jmh/code/hydro"
    echo "   cargo run -p dfir_rs --example chat_websocket -- --name 'ChatServer' --role server --address 127.0.0.1:3000"
fi

# Check if WASM files are accessible
echo "🔍 Checking WASM files..."
if curl -s -o /dev/null -w "%{http_code}" http://localhost:8081/examples/websocket_chat/pkg/website_playground.js | grep -q "200"; then
    echo "✅ WASM JavaScript bindings accessible"
else
    echo "❌ WASM files not found. Rebuilding..."
    ./build.sh
fi

if curl -s -o /dev/null -w "%{http_code}" http://localhost:8081/examples/websocket_chat/pkg/website_playground_bg.wasm | grep -q "200"; then
    echo "✅ WASM binary accessible"
else
    echo "❌ WASM binary not accessible"
fi

echo ""
echo "🚀 Ready to test!"
echo "1. Open browser: http://localhost:8081/examples/websocket_chat/"
echo "2. Use server URL: ws://localhost:3000"
echo "3. Enter any username and click Connect"
echo ""
echo "🔥 Pro tip: Open multiple tabs to test multi-user chat!"
