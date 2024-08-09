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

# Check if a project name was provided
if [ -z "$1" ]; then
    styled_echo $RED "❌ No project name provided. Please provide a project name."
    exit 1
fi

PROJECT_NAME=$1

# ASCII Art
echo "
${CYAN}
██╗ ██████╗██████╗     ███████╗██╗   ██╗███╗   ███╗    ██████╗ ██████╗ ██╗██████╗  ██████╗ ███████╗
██║██╔════╝██╔══██╗    ██╔════╝██║   ██║████╗ ████║    ██╔══██╗██╔══██╗██║██╔══██╗██╔═══██╗██╔════╝
██║██║     ██████╔╝    ███████╗██║   ██║██╔████╔██║    ██║  ██║██████╔╝██║██║  ██║██║   ██║███████╗
██║██║     ██╔══██╗    ╚════██║██║   ██║██║╚██╔╝██║    ██║  ██║██╔══██╗██║██║  ██║██║   ██║╚════██║
██║╚██████╗██║  ██║    ███████║╚██████╔╝██║ ╚═╝ ██║    ██████╔╝██║  ██║██║██████╔╝╚██████╔╝███████║
╚═╝ ╚═════╝╚═╝  ╚═╝    ╚══════╝ ╚═════╝ ╚═╝     ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚═╝╚═════╝  ╚═════╝ ╚══════╝
${RESET}"

styled_echo $YELLOW "🚀 Launching your ICP-EVM starter kit 🚀"
echo

# Create a new project directory
styled_echo $MAGENTA "🏗️  Crafting your new project space..."
mkdir $PROJECT_NAME
check_status
cd $PROJECT_NAME

# Clone the template repository into the specified project name directory
styled_echo $BLUE "🧬 Creating the template into $PROJECT_NAME..."
git clone https://github.com/Stephen-Kimoi/icp-evm-rust-bridge.git ./$PROJECT_NAME
check_status
styled_echo $GREEN "Template created succesfully into $PROJECT_NAME."

# Remove the .git directory to remove commit history and remote connection
rm -rf .git

# Initialize a new Git repository
styled_echo $BLUE "🔧 Initializing a new Git repository..."
git init
check_status
styled_echo $GREEN "✅ New Git repository initialized! Start fresh with your commits."
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
styled_echo $YELLOW "🚀 Deploying the evm_rpc canister locally..."
dfx deploy evm_rpc
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
