#!/bin/sh

# Build the frontend
echo "Building frontend..."
cd frontend
pnpm build
mv dist ../backend/resources/
cd ..

# Build the backend
echo "Building backend..."
cd backend
cargo build --release