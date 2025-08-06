export interface UserData {
  id: number;
  name: string;
  email: string;
}

export interface UserNotification {
  event_type: string;
  message: string;
  timestamp: string;
  user_data: UserData;
}

export interface WsMessage {
  user: string;
  message: string;
  timestamp: string;
}

export type NotificationMessage = 
  | { type: 'user_notification'; data: UserNotification }
  | { type: 'ws_message'; data: WsMessage }
  | { type: 'connected' }
  | { type: 'disconnected' }
  | { type: 'error'; message: string };

export interface WebSocketHookReturn {
  messages: NotificationMessage[];
  connected: boolean;
  autoReconnect: boolean;
  setAutoReconnect: (enabled: boolean) => void;
  clearMessages: () => void;
}
