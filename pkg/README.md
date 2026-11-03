# Comparateur de crédits immobiliers

Application locale Vue.js et Rust pour enregistrer des projets, gérer leurs offres et comparer leur coût.

## Architecture

- **Navigateur** : Vue 3 pilote l'interface et charge le module Rust compilé en WebAssembly depuis `pkg/`. Le WASM calcule les mensualités et classe les offres.
- **Serveur local** : Axum sert l'application et le module WASM, et expose l'API REST de persistance.
- **Base de données** : SQLite (`mortgage_calculator.db`) est lue et modifiée par le serveur Rust natif.
- **Documentation** : Utoipa génère le document OpenAPI depuis les routes et les modèles Rust. Swagger UI l'affiche automatiquement.

Le WASM s'exécute dans le sandbox du navigateur : il ne peut pas ouvrir le fichier SQLite du serveur. Les opérations de sauvegarde passent donc par l'API Axum, qui accède à la base locale. Les calculs, eux, sont effectués dans le navigateur par le WASM.

## Démarrage

Prérequis : Rust/Cargo, `wasm-pack`, Bash et `ss` (iproute2).

```bash
./start.sh
```

Le script lit le host et le port dans `config.ini`. Si le port est occupé, il essaie les ports suivants jusqu'à en trouver un de libre. Il compile ensuite le module WASM dans `pkg/`, compile le serveur natif et le lance sur l'adresse sélectionnée.

La première compilation nécessite les dépendances Rust du projet. Les feuilles de style et Vue.js sont chargés depuis les CDN configurés dans `index.html`.

## Documentation API

- Swagger UI généré : <http://localhost:8081/swagger-ui>
- Spécification OpenAPI JSON : <http://localhost:8081/api/openapi.json>
- Guide statique complémentaire : <http://localhost:8081/api-docs.html>

Les routes documentées incluent la gestion des projets et des offres, ainsi que l'endpoint de comparaison REST. L'interface de comparaison utilise le module WASM directement.

## API REST

| Méthode | Route | Description |
|---|---|---|
| `GET` | `/api/projects` | Lister les projets |
| `POST` | `/api/projects` | Créer un projet |
| `GET` | `/api/projects/{id}` | Lire un projet |
| `DELETE` | `/api/projects/{id}` | Supprimer un projet et ses offres |
| `GET` | `/api/projects/{id}/offers` | Lister les offres d'un projet |
| `PUT` | `/api/projects/{id}/offers` | Enregistrer les modifications et la liste complète des offres d'un projet |
| `POST` | `/api/offers` | Ajouter une offre |
| `PUT` | `/api/offers/{id}/name` | Modifier le nom d'une offre |
| `DELETE` | `/api/offers/{id}` | Supprimer une offre |
| `POST` | `/api/compare` | Comparer des offres côté serveur |

## Compilation manuelle

```bash
wasm-pack build --target web --out-dir pkg --release -- --features wasm
cargo build --bin server --features server
./target/debug/server
```

La base SQLite est créée automatiquement dans le répertoire du projet au premier démarrage.
