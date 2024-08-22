#!/bin/sh

# Build the frontend
echo "Building frontend..."
rm -rf frontend/packages/app/public/esm
ln -s ../resources/esm frontend/packages/app/public/esm
cd frontend
pnpm build
mv dist ../backend/resources/
cd ..

# Build the backend
echo "Building backend..."
cd backend
cargo build --release