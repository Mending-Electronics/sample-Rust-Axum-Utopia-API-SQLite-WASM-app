#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "$0")"

config_file="config.ini"
if [[ ! -r "$config_file" ]]; then
    echo "Erreur : fichier de configuration illisible : $config_file" >&2
    exit 1
fi

trim() {
    local value=$1
    value="${value#"${value%%[![:space:]]*}"}"
    value="${value%"${value##*[![:space:]]}"}"
    printf '%s' "$value"
}

host=""
port=""
while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%%#*}"
    line="${line%%;*}"
    line=$(trim "$line")
    [[ -z "$line" || "$line" == \[*\] || "$line" != *=* ]] && continue

    key=$(trim "${line%%=*}")
    value=$(trim "${line#*=}")
    case "$key" in
        Host) host=$value ;;
        Port) port=$value ;;
    esac
done < "$config_file"

if [[ -z "$host" || "$host" == *[[:space:]]* ]]; then
    echo "Erreur : Host manquant ou invalide dans $config_file" >&2
    exit 1
fi

if [[ ! "$port" =~ ^[0-9]{1,5}$ ]] || ((10#$port < 1 || 10#$port > 65535)); then
    echo "Erreur : Port manquant ou invalide dans $config_file (valeur attendue : 1-65535)" >&2
    exit 1
fi
port=$((10#$port))

if ! command -v ss >/dev/null 2>&1; then
    echo "Erreur : la commande ss est nécessaire pour vérifier si un port est disponible." >&2
    exit 1
fi

configured_port=$port
while true; do
    if ! listeners=$(ss -H -ltn "sport = :$port"); then
        echo "Erreur : impossible de vérifier la disponibilité du port $port." >&2
        exit 1
    fi
    [[ -z "$listeners" ]] && break

    if ((port == 65535)); then
        echo "Erreur : aucun port disponible à partir du port configuré $configured_port." >&2
        exit 1
    fi
    port=$((port + 1))
done

if ((port != configured_port)); then
    echo "⚠️ Le port $configured_port est occupé, utilisation du port $port."
fi

echo "🔨 Compilation du backend Rust/WASM..."
wasm-pack build --target web --out-dir pkg --release -- --features wasm

echo "🔨 Compilation du serveur Rust + SQLite..."
cargo build --bin server --features server

echo "🚀 Démarrage demandé sur http://$host:$port"
echo "📚 Swagger UI : http://$host:$port/swagger-ui"
echo "📄 OpenAPI JSON : http://$host:$port/api/openapi.json"
exec ./target/debug/server "$host" "$port"
