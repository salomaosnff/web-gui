#!/bin/bash

export LENZ_BASE_URL="http://localhost:5173"

# Inicia o backend e armazena o PID
rm -rf frontend/packages/app/public/esm

ls -la ./resources/esm

ln -s ../../../../resources/esm frontend/packages/app/public/esm
cd backend
cargo watch -x "run" &
PID_RUST=$!

# Inicia o frontend e armazena o PID
cd ../frontend
pnpm dev &
PID_VITE=$!

# Função para lidar com SIGINT (Ctrl+C)
cleanup() {
  echo "\nEncerrando processos..."
  kill $PID_RUST $PID_VITE
  wait $PID_RUST $PID_VITE
  exit 0
}

# Captura o SIGINT (Ctrl+C) e executa a função cleanup
trap cleanup SIGINT

# Aguarda os processos filhos
wait $PID_RUST $PID_VITE
