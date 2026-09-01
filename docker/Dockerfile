# syntax=docker/dockerfile:1

# ---------- Étage de build : compile tout le workspace une seule fois ----------
FROM rust:1-slim-bookworm AS builder

# pkg-config + libssl-dev nécessaires pour compiler reqwest (OpenSSL natif).
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# On copie tout le workspace. Le cache Docker recompile seulement si le code change.
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY services ./services
COPY ui/src-tauri ./ui/src-tauri

# On ne build que les 4 services backend (pas l'UI Tauri, non conteneurisable).
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release \
        -p weather-service \
        -p movie-service \
        -p recommendation-service \
        -p web-server \
    && mkdir -p /out \
    && cp target/release/weather-service \
          target/release/movie-service \
          target/release/recommendation-service \
          target/release/web-server \
          /out/

# ---------- Étage runtime : image légère, un seul binaire par conteneur ----------
FROM debian:bookworm-slim AS runtime

# ca-certificates : SANS lui, tout appel HTTPS (TMDb, Open-Meteo) échoue au
# handshake TLS. libssl3 : runtime OpenSSL requis par reqwest.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Répertoire pour la base SQLite (monté comme volume par docker-compose).
RUN mkdir -p /data

WORKDIR /app
COPY --from=builder /out/ /usr/local/bin/

# Le binaire à lancer est choisi via `command:` dans docker-compose.yml.
