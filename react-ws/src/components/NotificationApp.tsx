import React from 'react';
import { useWebSocket } from '../hooks/useWebSocket';
import { NotificationItem } from './NotificationItem';
import './NotificationApp.css';

const WS_URL = 'ws://localhost:3000/ws';

export const NotificationApp: React.FC = () => {
  const { messages, connected, autoReconnect, setAutoReconnect, clearMessages } = useWebSocket(WS_URL);

  const handleToggleReconnect = (e: React.ChangeEvent<HTMLInputElement>) => {
    setAutoReconnect(e.target.checked);
  };

  const handleClearMessages = () => {
    clearMessages();
  };

  return (
    <div className="notification-app">
      <header className="header">
        <h1>🔔 WebSocket Notifications (React)</h1>
        <div className="controls">
          <div className={`status ${connected ? 'connected' : 'disconnected'}`}>
            {connected ? '🟢 Connected' : '🔴 Disconnected'}
          </div>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={autoReconnect}
              onChange={handleToggleReconnect}
            />
            Auto-reconnect
          </label>
          <button onClick={handleClearMessages} className="clear-btn">
            🗑️ Clear
          </button>
        </div>
      </header>

      <main className="notifications">
        <div className="info-bar">
          <span>Total messages: {messages.length}</span>
          <span>WebSocket URL: {WS_URL}</span>
        </div>

        <div className="messages-container">
          {messages.length === 0 ? (
            <div className="empty-state">
              <p>🎯 Waiting for notifications...</p>
              <small>Create or delete users in the backend to see real-time notifications here.</small>
            </div>
          ) : (
            <div className="messages-list">
              {messages
                .slice()
                .reverse()
                .map((msg, index) => (
                  <NotificationItem
                    key={messages.length - index}
                    message={msg}
                    index={index}
                  />
                ))}
            </div>
          )}
        </div>
      </main>
    </div>
  );
};
