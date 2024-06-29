#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status.

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
        styled_echo $RED "❌ Last command failed. Exiting."
        exit 1
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
styled_echo $BLUE "🧬 Cloning the magical template repository..."
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
styled_echo $YELLOW "🌐 Igniting the local replica..."
dfx start --background
check_status
styled_echo $GREEN "✅ Local replica is alive and kicking!"
echo

# Deploy the backend canister
styled_echo $MAGENTA "🚀 Deploying the backend canister..."
./did.sh && dfx deploy backend 
check_status
styled_echo $GREEN "✅ Backend canister successfully deployed!"
echo

# Deploy the frontend canister
styled_echo $MAGENTA "🚀 Deploying the frontend canister..."
dfx deploy frontend
check_status
styled_echo $GREEN "✅ Frontend canister successfully deployed!"
echo

styled_echo $RED "🎉🎊 Congratulations! 🎊🎉"
styled_echo $GREEN "Your ICP-EVM integration project is now ready to conquer the decentralized universe!"
echo
styled_echo $YELLOW "May your code be bug-free and your transactions swift! Happy coding! 🖥️💻🚀"