import React from 'react';
import { NotificationMessage } from '../types/notifications';

interface NotificationItemProps {
  message: NotificationMessage;
  index: number;
}

const formatTime = (timestamp: string): string => {
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString();
  } catch {
    return timestamp.substring(0, 8);
  }
};

export const NotificationItem: React.FC<NotificationItemProps> = ({ message, index }) => {
  switch (message.type) {
    case 'user_notification':
      const notification = message.data;
      const eventColor = notification.event_type === 'user_created' ? 'success' : 
                        notification.event_type === 'user_deleted' ? 'warning' : 'info';
      
      return (
        <div className={`message notification ${eventColor}`}>
          <div className="message-header">
            <span className="event-type">
              {notification.event_type === 'user_created' ? '👤➕ User Created' :
               notification.event_type === 'user_deleted' ? '👤🗑️ User Deleted' :
               notification.event_type}
            </span>
            <time className="timestamp">
              {formatTime(notification.timestamp)}
            </time>
          </div>
          <div className="message-content">
            <div className="notification-message">
              {notification.message}
            </div>
            <div className="user-details">
              <strong>{notification.user_data.name}</strong>
              <span className="email">({notification.user_data.email})</span>
              <span className="user-id">ID: {notification.user_data.id}</span>
            </div>
          </div>
        </div>
      );

    case 'ws_message':
      const wsMsg = message.data;
      return (
        <div className="message ws-message">
          <div className="message-header">
            <span className="event-type">💬 Message</span>
            <time className="timestamp">
              {formatTime(wsMsg.timestamp)}
            </time>
          </div>
          <div className="message-content">
            <div className="user-name">{wsMsg.user}</div>
            <div className="message-text">{wsMsg.message}</div>
          </div>
        </div>
      );

    case 'connected':
      return (
        <div className="message system success">
          <div className="message-content">
            🟢 Connected to WebSocket server
          </div>
        </div>
      );

    case 'disconnected':
      return (
        <div className="message system warning">
          <div className="message-content">
            🔴 Disconnected from WebSocket server
          </div>
        </div>
      );

    case 'error':
      return (
        <div className="message system error">
          <div className="message-content">
            ❌ Error: {message.message}
          </div>
        </div>
      );

    default:
      return null;
  }
};
