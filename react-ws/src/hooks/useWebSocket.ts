import { useState, useEffect, useCallback, useRef } from 'react';
import { NotificationMessage, UserNotification, WsMessage, WebSocketHookReturn } from '../types/notifications';

const MAX_MESSAGES = 100;

export const useWebSocket = (url: string): WebSocketHookReturn => {
  const [messages, setMessages] = useState<NotificationMessage[]>([]);
  const [connected, setConnected] = useState(false);
  const [autoReconnect, setAutoReconnect] = useState(true);
  
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);

  const addMessage = useCallback((message: NotificationMessage) => {
    setMessages((prevMessages: NotificationMessage[]) => {
      const newMessages = [...prevMessages, message];
      if (newMessages.length > MAX_MESSAGES) {
        newMessages.shift();
      }
      return newMessages;
    });
  }, []);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    console.log('Connecting to WebSocket:', url);
    
    try {
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        console.log('WebSocket connected');
        setConnected(true);
        addMessage({ type: 'connected' });
        
        // Clear any existing reconnect timeout
        if (reconnectTimeoutRef.current) {
          clearTimeout(reconnectTimeoutRef.current);
          reconnectTimeoutRef.current = null;
        }
      };

      ws.onmessage = (event) => {
        const text = event.data;
        console.log('Received message:', text);

        try {
          // Try to parse as UserNotification first
          const userNotification = JSON.parse(text) as UserNotification;
          if (userNotification.event_type && userNotification.user_data) {
            addMessage({ type: 'user_notification', data: userNotification });
            return;
          }
        } catch {
          // Not a UserNotification, try WsMessage
        }

        try {
          const wsMessage = JSON.parse(text) as WsMessage;
          if (wsMessage.user && wsMessage.message) {
            addMessage({ type: 'ws_message', data: wsMessage });
            return;
          }
        } catch {
          // Not a WsMessage either
          console.warn('Could not parse message:', text);
        }
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setConnected(false);
        addMessage({ type: 'disconnected' });

        // Auto-reconnect if enabled
        if (autoReconnect) {
          console.log('Attempting to reconnect in 3 seconds...');
          reconnectTimeoutRef.current = window.setTimeout(() => {
            connect();
          }, 3000);
        }
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        setConnected(false);
        addMessage({ type: 'error', message: 'Connection error' });
      };

    } catch (error) {
      console.error('Failed to create WebSocket:', error);
      addMessage({ type: 'error', message: 'Failed to create WebSocket' });
    }
  }, [url, autoReconnect, addMessage]);

  useEffect(() => {
    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [connect]);

  useEffect(() => {
    // Handle auto-reconnect setting changes
    if (!autoReconnect && reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
  }, [autoReconnect]);

  const clearMessages = useCallback(() => {
    setMessages([]);
  }, []);

  return {
    messages,
    connected,
    autoReconnect,
    setAutoReconnect,
    clearMessages
  };
};
