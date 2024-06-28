# ICP-EVM Integration Starter Template

This template provides a seamless integration between Internet Computer Protocol (ICP) canisters and Ethereum Virtual Machine (EVM) based smart contracts. It uses Rust for the backend canister and includes a simple Solidity smart contract for demonstration.

It has been built on top of the [evm rpc rust](https://github.com/fxgst/evm-rpc-rust/tree/main) template by [Elias Datler](https://github.com/fxgst)

## Project Structure

```
/
├── backend/
│   └── src/
│       ├── eth_call.rs
│       ├── evm_rpc.rs
│       └── lib.rs
├── smart_contract/
│   └── contracts/
│       └── Counter.sol
└── src/
    └── (frontend files)
```

## Backend (Rust Canister)

### eth_call.rs

This file contains the core functionality for interacting with EVM-based smart contracts.

Key components:
- `call_smart_contract`: A function that handles both read and write operations to the smart contract.
- `sign_transaction`: Handles the signing of transactions for write operations.
- `send_raw_transaction`: Sends the signed transaction to the EVM network.
- Helper functions for conversion between different data formats.

### evm_rpc.rs

This file defines the structures and implementations for interacting with EVM RPC services.

Key components:
- Various structs and enums representing RPC requests and responses.
- `EvmRpcCanister`: An implementation that provides methods for common Ethereum RPC calls.
- `Service`: A struct that wraps the canister ID for making RPC calls.

### lib.rs

This is the main entry point for the canister, containing the public functions that can be called.

Key functions:
- `get_latest_ethereum_block`: Retrieves the latest block from the Ethereum network.
- `get_canister_eth_address`: Derives the Ethereum address for the canister.
- `call_increase_count`: Increases the counter in the smart contract.
- `get_count`: Retrieves the current count from the smart contract.
- `call_decrease_count`: Decreases the counter in the smart contract.

## Smart Contract (Solidity)

### Counter.sol

A simple smart contract that implements a counter with functions to increase, decrease, and get the current count.

## Quick Start

To get started with this template, run the following command:

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/https://github.com/Stephen-Kimoi/icp-evm-rust-bridge/main/install_and_deploy.sh)"
```

This script will:
1. Create a new project
2. Clone this template into the new project
3. Install dependencies
4. Start a local replica
5. Deploy the canister

After running the script, your project will be set up and ready to use!

## Manual Setup

If you prefer to set up the project manually:

1. Clone this repository
2. Install dependencies with `npm install`
3. Start a local replica with `dfx start --background`
4. Deploy the canister with `dfx deploy`

## Usage

[Include instructions on how to use the template, call canister functions, etc.]

## Contributing

[Include instructions for contributing to the template]

## License

[Include license information]