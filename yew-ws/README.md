# WebSocket Notifications Frontend 🔔

Frontend Yew minimal pour afficher les notifications WebSocket en temps réel.

## 🚀 Démarrage Rapide

### 1. Construire le frontend
```bash
cd yew-ws
cargo install trunk
rustup target add wasm32-unknown-unknown
trunk build --release
```

### 2. Ou utiliser le script
```bash
# Windows
./build-notifications.bat

# Linux/macOS  
chmod +x build-notifications.sh && ./build-notifications.sh
```

### 3. Démarrer le backend
```bash
# Depuis la racine du projet
cargo run
```

### 4. Accéder au frontend
- **Frontend notifications**: http://localhost:3000/notifications/
- **WebSocket endpoint**: ws://localhost:3000/ws

## ✨ Fonctionnalités

### Notifications en temps réel
- ✅ **Notifications d'utilisateurs** : Création/suppression d'utilisateurs
- ✅ **Messages WebSocket** : Messages chat/génériques  
- ✅ **Messages système** : Connexion/déconnexion/erreurs
- ✅ **Auto-reconnexion** : Reconnexion automatique en cas de perte
- ✅ **Historique** : Affichage des 100 derniers messages
- ✅ **Design responsive** : Interface adaptative mobile/desktop

### Interface utilisateur
- 🎨 **Design moderne** avec dégradés et animations
- 🔄 **Statut de connexion** en temps réel  
- 🕒 **Timestamps** formatés
- 🗑️ **Effacement** de l'historique
- ⚙️ **Toggle auto-reconnexion**

### Types de notifications
1. **👤➕ User Created** - Nouvel utilisateur créé
2. **👤🗑️ User Deleted** - Utilisateur supprimé  
3. **💬 Message** - Message WebSocket générique
4. **🟢 Connected** - Connexion établie
5. **🔴 Disconnected** - Connexion perdue
6. **❌ Error** - Erreur de connexion

## 🧪 Test

Pour tester les notifications :

1. **Démarrer le frontend notifications**
2. **Depuis une autre fenêtre**, utiliser l'API REST :

```bash
# Créer un utilisateur (génère une notification)
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Test User","email":"test@example.com"}'

# Supprimer un utilisateur (génère une notification)  
curl -X DELETE http://localhost:3000/users/1
```

Les notifications apparaîtront instantanément dans le frontend !

## 🏗️ Architecture

```
yew-ws/
├── src/
│   ├── lib.rs          # Point d'entrée
│   ├── app.rs          # Composant principal  
│   ├── models.rs       # Modèles de données
│   └── websocket.rs    # Service WebSocket (optionnel)
├── index.html          # Template HTML + CSS
├── Cargo.toml          # Dépendances Yew
└── dist/               # Build de production
```

## 🎨 Personnalisation

Le CSS est intégré dans `index.html` pour faciliter la customisation :
- **Couleurs** : Modifiez les gradients CSS
- **Animations** : Ajustez les transitions et keyframes  
- **Layout** : Changez la disposition responsive
- **Thème** : Créez vos propres styles de messages
