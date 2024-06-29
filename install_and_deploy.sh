#!/bin/bash

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
RESET='\033[0m'

# Function for styled echo
styled_echo() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${RESET}"
}

# Function to check last command status
check_status() {
    if [ $? -ne 0 ]; then
        styled_echo $RED "❌ Last command failed. Proceeding with the next command"
    fi
}

# ASCII Art
echo "
${CYAN}░▀█▀░█▀▀░█▀█░░░░░█▀▀░█░█░█▄█░░░█▀▄░█▀▄░▀█▀░█▀▄░█▀▀░█▀▀
░░█░░█░░░█▀█░░░░░█▀▀░▀▄▀░█░█░░░█▀▄░█▀▄░░█░░█░█░█░█░█▀▀
░░▀░░▀▀▀░▀░▀░▀▀▀░▀▀▀░░▀░░▀░▀░░░▀▀░░▀░▀░░▀░░▀▀░░▀▀▀░▀▀▀${RESET}"

styled_echo $YELLOW "🚀 Launching your ICP-EVM starter kit 🚀"
echo

# Create a new project
styled_echo $MAGENTA "🏗️  Crafting your new project space..."
echo

# Clone the template repository
styled_echo $BLUE "🧬 Cloning the template repository..."
git clone https://github.com/Stephen-Kimoi/icp-evm-rust-bridge.git
check_status
styled_echo $GREEN "Repository cloned successfully."

styled_echo $BLUE "Copying files from cloned repository..."
cp -R icp-evm-rust-bridge/* .
check_status
styled_echo $GREEN "✅ Template successfully cloned and files copied!"
echo

# Install dependencies
styled_echo $CYAN "📦 Installing project dependencies..."
npm install
check_status
styled_echo $GREEN "✅ Dependencies successfully installed!"
echo

# Start the local replica
styled_echo $YELLOW "🌐 Starting the local replica..."
dfx start --background
check_status
styled_echo $GREEN "✅ Local replica is alive and kicking!"
echo

# Locally deploy the `evm_rpc` canister
styled_echo $YELLOW "🚀 Pulling and deploying the evm_rpc canister locally..."
dfx deps pull
dfx deps init evm_rpc --argument '(record { nodesInSubnet = 28 })'
check_status
styled_echo $GREEN "✅ EVM RPC canister deployed locally..."
echo

# Deploy the canisters
styled_echo $MAGENTA "🚀 Deploying the backend canister..."
./did.sh && dfx deploy 
check_status
styled_echo $GREEN "✅ Backend canister successfully deployed!"
echo

styled_echo $RED "🎉🎊 Congratulations! 🎊🎉"
styled_echo $GREEN "Your ICP-EVM integration project is now ready!"
echo
styled_echo $YELLOW "To get started, check out the official documentation:"
styled_echo $CYAN "https://github.com/Stephen-Kimoi/icp-evm-rust-bridge#icp-evm-integration-starter-template"
echo
styled_echo $YELLOW "Happy coding! 🖥️💻🚀"