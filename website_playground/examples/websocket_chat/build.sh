#!/bin/bash

set -e

echo "🌊 Building DFIR WebSocket Chat (WASM)"
echo "======================================="

# Navigate to the website_playground directory
cd "$(dirname "$0")/../../"
echo "📁 Working directory: $(pwd)"

# Check prerequisites
echo "🔍 Checking prerequisites..."

if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo "❌ WASM target not installed. Installing..."
    rustup target add wasm32-unknown-unknown
else
    echo "✅ WASM target installed"
fi

if ! command -v wasm-bindgen &> /dev/null; then
    echo "❌ wasm-bindgen not found. Installing..."
    cargo install wasm-bindgen-cli
else
    echo "✅ wasm-bindgen available"
fi

# Build the WASM module
echo "🔨 Building WASM module..."
cargo build --target wasm32-unknown-unknown --release

# Generate JavaScript bindings
echo "🔗 Generating JavaScript bindings..."
wasm-bindgen --out-dir pkg --target web ../target/wasm32-unknown-unknown/release/website_playground.wasm

# Copy necessary files to examples directory
echo "📋 Setting up example files..."
mkdir -p examples/websocket_chat/pkg
cp pkg/website_playground.js examples/websocket_chat/pkg/
cp pkg/website_playground_bg.wasm examples/websocket_chat/pkg/

echo "✅ Build complete!"
echo ""
echo "🚀 To run the WebSocket Chat:"
echo "1. Start the WebSocket server:"
echo "   cd ../dfir_rs/examples/chat_websocket"
echo "   cargo run -- --role server --address 127.0.0.1:3000"
echo ""
echo "2. Serve the web application:"
echo "   cd website_playground"
echo "   python3 -m http.server 8081"
echo ""
echo "3. Open browser:"
echo "   http://localhost:8081/examples/websocket_chat/"
echo ""
echo "🎉 Happy chatting with DFIR!"
