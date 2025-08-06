# React WebSocket Notifications Frontend 🔔

React TypeScript frontend to display WebSocket notifications in real-time.

## 🚀 Quick Start

### Prerequisites
- Node.js 16+ installed: https://nodejs.org/
- npm or yarn package manager

### 1. Install dependencies
```bash
cd react-ws
npm install
```

### 2. Start development server
```bash
npm start
```
This will start the React development server on http://localhost:3001

### 3. Build for production
```bash
npm run build
```
This creates optimized production files in the `build/` directory.

### 4. Start the backend
```bash
# From the root project directory
cargo run
```

### 5. Access the applications
- **React frontend (dev)**: http://localhost:3001/
- **React frontend (prod)**: http://localhost:3000/react/ (after build)
- **Backend API**: http://localhost:3000/
- **WebSocket endpoint**: ws://localhost:3000/ws

## ✨ Features

### Real-time notifications
- ✅ **User notifications**: Creation/deletion of users
- ✅ **WebSocket messages**: Chat/generic messages  
- ✅ **System messages**: Connection/disconnection/errors
- ✅ **Auto-reconnection**: Automatic reconnection on connection loss
- ✅ **Message history**: Display of last 100 messages
- ✅ **Responsive design**: Mobile/desktop adaptive interface

### User interface
- 🎨 **Modern design** with gradients and animations
- 🔄 **Real-time connection status**  
- 🕒 **Formatted timestamps**
- 🗑️ **Message history clearing**
- ⚙️ **Auto-reconnection toggle**

## 🛠️ Available Scripts

- `npm start` - Start development server (port 3001)
- `npm run build` - Build for production
- `npm test` - Run tests
- `npm run eject` - Eject from Create React App

## 🏗️ Project Structure

```
react-ws/
├── public/
│   └── index.html           # HTML template
├── src/
│   ├── components/
│   │   ├── NotificationApp.tsx    # Main app component
│   │   ├── NotificationApp.css    # App styles
│   │   └── NotificationItem.tsx   # Individual notification
│   ├── hooks/
│   │   └── useWebSocket.ts        # WebSocket connection hook
│   ├── types/
│   │   └── notifications.ts       # TypeScript interfaces
│   ├── App.tsx              # Root component
│   └── index.tsx            # Entry point
├── package.json             # Dependencies & scripts
└── tsconfig.json            # TypeScript config
```

## 🔧 Backend Integration

The React app connects to the same WebSocket endpoint as the Yew app:
- **WebSocket URL**: `ws://localhost:3000/ws`
- **Expected message formats**:
  - User notifications (JSON with `event_type`, `user_data`, etc.)
  - WebSocket messages (JSON with `user`, `message`, `timestamp`)

## 🎨 Technologies Used

- **React 18** with TypeScript
- **Custom hooks** for WebSocket management
- **CSS3** with modern features (gradients, flexbox, grid)
- **WebSocket API** for real-time communication

## 📱 Responsive Design

The application is fully responsive and works on:
- 📱 Mobile devices (320px+)
- 💻 Tablets (768px+)
- 🖥️ Desktop computers (1024px+)

## 🔄 Comparison with Yew Frontend

Both frontends provide identical functionality:

| Feature | React | Yew |
|---------|--------|-----|
| Language | TypeScript | Rust |
| Bundle Size | ~500KB | ~200KB |
| Development Speed | Fast | Medium |
| Runtime Performance | Fast | Faster |
| Ecosystem | Large | Growing |
| Learning Curve | Easy | Steep |
| Hot Reload | Excellent | Limited |
| Debugging | Excellent | Limited |

## 🚀 Build Scripts

Use the root-level build scripts for convenience:

**Windows:**
```bash
.\build-react.bat
```

**Linux/macOS:**
```bash
./build-react.sh
```

## 🌐 URLs Summary

- **Development**: http://localhost:3001/ (React dev server)
- **Production**: http://localhost:3000/react/ (served by Rust backend)
- **WebSocket**: ws://localhost:3000/ws
- **Yew comparison**: http://localhost:3000/notifications/
