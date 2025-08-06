# Yew WebSocket Notifications

Cette application Yew affiche en temps réel les notifications reçues via WebSocket depuis le serveur Zevis.

## Fonctionnalités

- 🔄 Connexion automatique au WebSocket
- 📱 Interface responsive
- 🎨 Design moderne avec dégradés
- 🔄 Reconnexion automatique en cas de perte de connexion
- 🗑️ Possibilité de vider l'historique des messages
- 📊 Affichage du statut de connexion

## Build et déploiement

### Prérequis

- Rust avec `wasm32-unknown-unknown` target
- Trunk (installé automatiquement par les scripts de build)

```bash
# Installer le target wasm
rustup target add wasm32-unknown-unknown
```

### Build automatique

```bash
# Windows
./build-yew.bat

# Unix/Linux/macOS
./build-yew.sh
```

### Build manuel

```bash
cd yew-ws
trunk build --release --dist dist
```

### Développement

Pour le développement avec rechargement automatique :

```bash
cd yew-ws
trunk serve --open
```

## Utilisation

1. Lancez le serveur Zevis principal
2. Buildez l'application Yew avec les scripts fournis
3. Accédez à `http://localhost:3000/yew/`
4. L'application se connecte automatiquement au WebSocket sur `ws://localhost:3000/ws`

## Structure

- `src/app.rs` - Composant principal avec logique WebSocket
- `src/models.rs` - Structures de données pour les notifications
- `src/lib.rs` - Point d'entrée de l'application
- `dist/` - Fichiers générés (HTML, JS, WASM)

## Types de notifications supportés

- **User Created** - Notification de création d'utilisateur
- **User Deleted** - Notification de suppression d'utilisateur
- **WebSocket Messages** - Messages génériques via WebSocket
- **System Messages** - Messages de connexion/déconnexion/erreurs
