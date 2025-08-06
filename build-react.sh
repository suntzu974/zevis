#!/bin/bash
echo "Building React WebSocket Notifications Frontend..."

cd react-ws

echo "Installing dependencies..."
npm install

echo "Building React app for production..."
npm run build

echo "React build complete! Build files are in react-ws/build/"
echo "You can now access the React frontend at: http://localhost:3000/react/"

cd ..
