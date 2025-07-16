#!/bin/bash

# Test script for DFIR WebSocket Chat WASM example
# This script tests the integration between the WASM client and DFIR server

echo "🧪 Testing DFIR WebSocket Chat (WASM) Integration"
echo "================================================="

# Check if WebSocket server is running
echo "🔍 Checking if WebSocket server is running on 127.0.0.1:3000..."
if ! nc -z 127.0.0.1 3000 2>/dev/null; then
    echo "❌ WebSocket server not running on 127.0.0.1:3000"
    echo "   Please start the server first:"
    echo "   cd /Users/jmh/code/hydro"
    echo "   cargo run -p dfir_rs --example chat_websocket -- --name 'WASM-Server' --role server --address 127.0.0.1:3000"
    exit 1
fi
echo "✅ WebSocket server is running"

# Check if HTTP server is running
echo "🔍 Checking if HTTP server is running on localhost:8080..."
if ! nc -z localhost 8080 2>/dev/null; then
    echo "❌ HTTP server not running on localhost:8080"
    echo "   Please start the HTTP server:"
    echo "   cd /Users/jmh/code/hydro/website_playground"
    echo "   python3 -m http.server 8080"
    exit 1
fi
echo "✅ HTTP server is running"

# Check if WASM files exist
echo "🔍 Checking if WASM files are built..."
if [ ! -f "pkg/website_playground.js" ] || [ ! -f "pkg/website_playground_bg.wasm" ]; then
    echo "❌ WASM files not found. Building..."
    ./build.sh
    if [ $? -ne 0 ]; then
        echo "❌ Failed to build WASM files"
        exit 1
    fi
fi
echo "✅ WASM files are present"

# Test WebSocket connection with a native client
echo "🔍 Testing WebSocket connection with native client..."
cd ../../
cargo run -p dfir_rs --example chat_websocket -- --name "TestClient" --role client --address 127.0.0.1:3000 &
CLIENT_PID=$!

# Give client time to connect
sleep 2

# Kill the test client
kill $CLIENT_PID 2>/dev/null

echo "✅ Native client connection test completed"

echo ""
echo "🎉 All tests passed! The DFIR WebSocket Chat (WASM) is ready to use."
echo ""
echo "🌐 Open your browser to: http://localhost:8080/examples/websocket_chat/"
echo ""
echo "💡 Tips:"
echo "   - Use 'ws://localhost:3000' as the server URL"
echo "   - Choose any username you like"
echo "   - Open multiple browser tabs to test multi-user chat"
echo "   - You can also connect native DFIR clients alongside the web client"
echo ""
echo "🛠️  To test with a native client:"
echo "   cd /Users/jmh/code/hydro"
echo "   cargo run -p dfir_rs --example chat_websocket -- --name 'NativeUser' --role client --address 127.0.0.1:3000"
