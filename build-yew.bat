@echo off
echo Building Yew WebSocket Notifications App...

cd yew-ws

echo Installing trunk if not present...
cargo install trunk --quiet 2>nul || echo Trunk already installed

echo Building the Yew application...
trunk build --release --dist dist

if %ERRORLEVEL% EQU 0 (
    echo ✅ Yew app built successfully!
    echo 🌐 App available at http://localhost:3000/yew/
) else (
    echo ❌ Build failed
    exit /b 1
)

cd ..
