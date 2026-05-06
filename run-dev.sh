#!/bin/bash

if [ ! -f .env ]; then
    echo "No .env file!!"
    exit 1
fi
set -a
source .env
set +a

if [ ! -d ".venv" ]; then
    python3 -m venv .venv
    source .venv/bin/activate
    pip install nikos-safebooru-wrapper nikos-rule34-wrapper
else
    source .venv/bin/activate
fi

BINARY="./target/debug/nikos-discord-bot"
if [ ! -f "$BINARY" ]; then
    cargo build
else
    cargo build
fi

if [ -f "$BINARY" ]; then
    cargo run
else
    exit 1
fi