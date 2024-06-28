#!/bin/bash

# install_and_deploy.sh

# Create a new project
echo "Creating a new project..."

# Clone the template repository
echo "Cloning template repository..."
git clone https://github.com/Stephen-Kimoi/icp-evm-rust-bridge.git 
cp -R icp-evm-rust-bridge/* .

# Install dependencies
echo "Installing dependencies..."
npm install

# Start the local replica
echo "Starting local replica..."
dfx start --background

# Deploy the canister
echo "Deploying the canister..."
dfx deploy

echo "Setup complete! Your ICP-EVM integration project is ready."