#!/bin/bash
echo "Building Yew WebSocket Notifications App..."

cd yew-ws

echo "Installing trunk if not present..."
if ! command -v trunk &> /dev/null; then
    cargo install trunk
fi

echo "Building the Yew application..."
trunk build --release --dist dist

if [ $? -eq 0 ]; then
    echo "✅ Yew app built successfully!"
    echo "🌐 App available at http://localhost:3000/yew/"
else
    echo "❌ Build failed"
    exit 1
fi

cd ..
