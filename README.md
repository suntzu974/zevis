# Zevis - Application Axum avec PostgreSQL et WebSocket

Une application web Rust moderne utilisant Axum, PostgreSQL, Redis et WebSocket pour la communication en temps réel.

## 🚀 Fonctionnalités

- **API REST** pour la gestion des utilisateurs
- **Base de données PostgreSQL** avec SQLx pour la persistance
- **WebSocket** pour les notifications en temps réel
- **Redis** pour le broadcast des messages WebSocket
- **Interface web** de test interactive
- **Notifications automatiques** pour les opérations CRUD

## 📋 Prérequis

- [Rust](https://rustup.rs/) (dernière version stable)
- [Docker & Docker Compose](https://www.docker.com/get-started/)

## 🛠️ Installation

1. **Cloner le projet**
   ```bash
   git clone <repository-url>
   cd zevis
   ```

2. **Démarrer les services (PostgreSQL + Redis)**
   ```bash
   docker compose up -d
   ```

3. **Installer les dépendances Rust**
   ```bash
   cargo build
   ```

4. **Vérifier les variables d'environnement** (optionnel)
   ```bash
   # Fichier .env déjà configuré pour Docker local
   cp .env.example .env  # Si nécessaire
   ```

## 🏃 Exécution

1. **Démarrer l'application**
   ```bash
   cargo run
   ```

2. **Accéder à l'interface de test**
   - Interface web : http://127.0.0.1:3000/static/index.html
   - WebSocket : ws://127.0.0.1:3000/ws
   - Health check : http://127.0.0.1:3000/health

## 📡 API Endpoints

### Utilisateurs
- `GET /users` - Liste tous les utilisateurs
- `GET /users/:id` - Récupère un utilisateur par ID
- `POST /users` - Crée un nouvel utilisateur
- `DELETE /users/:id` - Supprime un utilisateur

### Cache (Redis)
- `GET /cache/:key` - Récupère une valeur du cache
- `POST /cache/:key` - Stocke une valeur dans le cache
- `DELETE /cache/:key` - Supprime une valeur du cache

### WebSocket
- `GET /ws` - Connexion WebSocket pour les notifications temps réel

### Système
- `GET /health` - Vérification de l'état des services

## 🔧 Exemples d'utilisation

### Créer un utilisateur
```bash
curl -X POST http://127.0.0.1:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice", "email": "alice@example.com"}'
```

### WebSocket avec JavaScript
```javascript
const ws = new WebSocket('ws://127.0.0.1:3000/ws');

ws.onmessage = (event) => {
    const notification = JSON.parse(event.data);
    console.log('Notification reçue:', notification);
};
```

## 🗄️ Structure de la base de données

### Table `users`
```sql
id SERIAL PRIMARY KEY,
name VARCHAR(255) NOT NULL,
email VARCHAR(255) NOT NULL UNIQUE,
created_at TIMESTAMPTZ DEFAULT NOW(),
updated_at TIMESTAMPTZ DEFAULT NOW()
```

### Table `user_events`
```sql
id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
event_type VARCHAR(50) NOT NULL,
user_id INTEGER,
user_data JSONB,
message TEXT,
created_at TIMESTAMPTZ DEFAULT NOW()
```

## 🔄 Notifications WebSocket

L'application envoie automatiquement des notifications via WebSocket lors de :
- Création d'utilisateur (`user_created`)
- Suppression d'utilisateur (`user_deleted`)

Format des notifications :
```json
{
  "id": "uuid",
  "event_type": "user_created",
  "user_data": {
    "id": 1,
    "name": "Alice",
    "email": "alice@example.com",
    "created_at": "2025-08-05T10:30:00Z",
    "updated_at": "2025-08-05T10:30:00Z"
  },
  "timestamp": "2025-08-05T10:30:00Z",
  "message": "Nouvel utilisateur créé: Alice (alice@example.com)"
}
```

## 🧪 Tests

Interface de test complète disponible à : http://127.0.0.1:3000/static/index.html

Fonctionnalités de test :
- Chat WebSocket en temps réel
- Création/suppression d'utilisateurs
- Visualisation des notifications
- Affichage avec styles différenciés

## 🛠️ Développement

### Migrations de base de données
```bash
# Créer une nouvelle migration
sqlx migrate add <nom_migration>

# Appliquer les migrations
sqlx migrate run
```

### Variables d'environnement
```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/zevis
REDIS_URL=redis://localhost:6379/
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
```

## 📦 Architecture

```
src/
├── main.rs           # Point d'entrée principal
├── migrations/       # Migrations SQL
├── static/          # Fichiers statiques (HTML, CSS, JS)
└── .env             # Configuration

Technologies utilisées:
- Axum (Framework web)
- SQLx (ORM PostgreSQL)
- Redis (Cache & WebSocket broadcast)
- Tokio (Runtime async)
- Serde (Sérialisation JSON)
```

## 🚀 Production

Pour déployer en production :

1. Configurer les URLs de production dans `.env`
2. Utiliser un gestionnaire de processus (systemd, PM2)
3. Configurer un reverse proxy (nginx)
4. Activer SSL/TLS
5. Configurer les logs et monitoring
