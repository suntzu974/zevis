@echo off
setlocal enabledelayedexpansion

echo 🔔 Building WebSocket Notifications Frontend...

REM Install trunk if not already installed
where trunk >nul 2>nul
if !errorlevel! neq 0 (
    echo Installing trunk...
    cargo install trunk
)

REM Add wasm32 target if not already added
rustup target add wasm32-unknown-unknown

REM Build the frontend
echo Building frontend...
trunk build --release

echo ✅ Frontend build complete!
echo 📁 Built files are in yew-ws/dist/
echo.
echo To serve the frontend:
echo   cd yew-ws && trunk serve --open --port 8080
echo.
echo Or copy dist/ contents to your backend's static folder
