mod store_transactions;
use candid::Principal;
// use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
// use eth_call::{ call_smart_contract, get_ecdsa_public_key };
use store_transactions::{store_transaction_hash, get_transaction_hashes}; 
// use ethers_core::{k256::elliptic_curve::{sec1::ToEncodedPoint, PublicKey}, types::U256, utils::keccak256};
use evm_rpc_canister_types::EvmRpcCanister;
// use k256::Secp256k1;
use alloy::{
    network::{Network, TxSigner}, providers::{Provider, ProviderBuilder}, signers::icp::IcpSigner, transports::icp::{EthSepoliaService, IcpConfig, RpcApi, RpcService}
};
// use ethers_core::types::TransactionRequest;
// use alloy_rpc_types_eth::TransactionRequest;
use alloy::network::Ethereum;
// use alloy::network::TransactionBuilder;
// use alloy_consensus::TxLegacy; 
use alloy::consensus::TxLegacy;
use alloy::consensus::Signed;
use alloy::consensus::SignableTransaction;
use alloy_rlp::Encodable;
use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, U256, TxKind};
use ic_cdk::api::management_canister::provisional::CanisterId;

pub const EVM_RPC_CANISTER_ID: Principal =
    Principal::from_slice(b"\x00\x00\x00\x00\x02\x30\x00\xCC\x01\x01"); // 7hfb6-caaaa-aaaar-qadga-cai
pub const EVM_RPC: EvmRpcCanister = EvmRpcCanister(EVM_RPC_CANISTER_ID);

fn get_rpc_service_sepolia() -> RpcService {
    // Use EVM RPC Canister with Alchemy for Sepolia
    // RpcService::EthSepolia(EthSepoliaService::Alchemy)
    
    // Removing the custom proxy configuration since it can lead to inconsistent results
    // when getting latest blocks across different nodes in the subnet
    RpcService::Custom(RpcApi {
        url: "https://ic-alloy-evm-rpc-proxy.kristofer-977.workers.dev/eth-sepolia".to_string(),
        headers: None,
    })
}

#[ic_cdk::update]
async fn get_latest_ethereum_block() -> Result<String, String> {
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);
    let result = provider.get_block_number().await;
    
    // Get the latest block
    match result {
        Ok(block) => Ok(block.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

// FUNCTIONS FOR CALLING THE SMART CONTRACT
const CONTRACT_ADDRESS: &str = "0xAed5d7b083ad30ad6B50f698427aD4907845AAc3";

const ABI_JSON: &str = r#"
   [
        {
            "inputs": [],
            "stateMutability": "nonpayable",
            "type": "constructor"
        },
        {
            "inputs": [],
            "name": "decreaseCount",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "getCount",
            "outputs": [
                {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "increaseCount",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]
"#;

fn get_abi() -> ethers_core::abi::Contract {
    serde_json::from_str::<ethers_core::abi::Contract>(ABI_JSON)
        .expect("Failed to parse ABI")
}

fn get_rpc_service() -> RpcService {
    // RpcService::EthSepolia(EthSepoliaService::Alchemy)
    RpcService::Custom(RpcApi {
        url: "https://ic-alloy-evm-rpc-proxy.kristofer-977.workers.dev/eth-sepolia".to_string(),
        headers: None,
    })
}

async fn create_icp_signer() -> IcpSigner {
    let key_name = "key_1".to_string(); // For mainnet deployment
    IcpSigner::new(vec![], &key_name, None).await.unwrap()
}

#[ic_cdk::update]
async fn get_canister_eth_address() -> String {
    let signer = create_icp_signer().await;
    signer.address().to_string()
}

#[ic_cdk::update]
async fn call_increase_count() -> Result<String, String> {
    let signer = create_icp_signer().await;
    let config = IcpConfig::new(get_rpc_service());
    let provider = ProviderBuilder::new().on_icp(config);
    let abi = get_abi();

    // Get the current nonce for the signer's address
    let nonce = provider.get_transaction_count(signer.address())
        .await
        .map_err(|e| format!("Failed to get nonce: {:?}", e))?;

    // Get the current gas price
    let gas_price = provider.get_gas_price()
        .await
        .map_err(|e| format!("Failed to get gas price: {:?}", e))?;

    // Calculate increased gas price (120%)
    let adjusted_gas_price = gas_price + (gas_price / 5); // Add 20%

    // Parse contract address using Address type
    let contract_addr = Address::parse_checksummed(CONTRACT_ADDRESS, None)
        .map_err(|e| format!("Failed to parse contract address: {:?}", e))?;

    let encoded_function_call = abi.function("increaseCount")
        .unwrap()
        .encode_input(&[])
        .unwrap();

    // Clone the encoded call data since we'll need it twice
    let encoded_call_data = encoded_function_call.clone();

    // Create the legacy transaction directly
    let mut tx = TxLegacy {
        nonce,
        gas_price: adjusted_gas_price,
        gas_limit: 200_000,
        to: TxKind::Call(contract_addr),
        value: U256::ZERO,
        input: encoded_function_call.into(),
        chain_id: Some(11155111_u64), // Sepolia chain ID
    };    

    // Sign and encode the transaction
    let signature = signer.sign_transaction(&mut tx)
        .await
        .map_err(|e| format!("Failed to sign transaction: {:?}", e))?;

    let hash = tx.signature_hash();
    let signed_tx = Signed::new_unchecked(tx, signature, hash);
    let tx_for_sending = signed_tx.tx();

    let mut encoded_tx = Vec::new();
    tx_for_sending.encode(&mut encoded_tx);

    // Try to send the transaction
    match provider.send_raw_transaction(&encoded_tx).await {
        Ok(pending_tx) => {
            let hash = pending_tx.tx_hash().to_string();
            store_transaction_hash(hash.clone());
            Ok(format!("Increased count. Transaction hash: {}", hash))
        },
        Err(e) => {
            // If first attempt fails, try one more time with higher gas price
            let higher_gas_price = adjusted_gas_price + (adjusted_gas_price / 2); // Add 50% more
            
            let mut retry_tx = TxLegacy {
                nonce,
                gas_price: higher_gas_price,
                gas_limit: 300_000, // Increase gas limit for retry
                to: TxKind::Call(contract_addr),
                value: U256::ZERO,
                input: encoded_call_data.into(),
                chain_id: Some(11155111_u64),
            };

            let retry_signature = signer.sign_transaction(&mut retry_tx)
                .await
                .map_err(|e| format!("Failed to sign retry transaction: {:?}", e))?;

            let retry_hash = retry_tx.signature_hash();
            let retry_signed_tx = Signed::new_unchecked(retry_tx, retry_signature, retry_hash);
            let retry_tx_for_sending = retry_signed_tx.tx();

            let mut retry_encoded_tx = Vec::new();
            retry_tx_for_sending.encode(&mut retry_encoded_tx);

            match provider.send_raw_transaction(&retry_encoded_tx).await {
                Ok(pending_tx) => {
                    let hash = pending_tx.tx_hash().to_string();
                    store_transaction_hash(hash.clone());
                    Ok(format!("Increased count on retry. Transaction hash: {}", hash))
                },
                Err(retry_e) => {
                    Err(format!("Transaction failed on both attempts. Initial error: {:?}, Retry error: {:?}", e, retry_e))
                }
            }
        }
    }
}

#[ic_cdk::update]
async fn get_count() -> Result<u64, String> {
    let config = IcpConfig::new(get_rpc_service());
    let provider = ProviderBuilder::new().on_icp(config);
    let abi = get_abi();

    let result = provider.call(
        &<Ethereum as Network>::TransactionRequest::default()
            .to(CONTRACT_ADDRESS.parse().unwrap())
            .input(abi.function("getCount")
                .unwrap()
                .encode_input(&[])
                .unwrap()
                .into())
    ).await;    

    match result {
        Ok(output) => {
            let decoded = abi.function("getCount")
                .unwrap()
                .decode_output(&output)
                .unwrap();
            Ok(decoded[0].clone().into_uint().unwrap().low_u64())
        },
        Err(e) => Err(format!("Failed to get count: {:?}", e))
    }
}

#[ic_cdk::update]
async fn call_decrease_count() -> Result<String, String> {
    // let signer = create_icp_signer().await;
    let config = IcpConfig::new(get_rpc_service());
    let provider = ProviderBuilder::new().on_icp(config);
    let abi = get_abi();

    let tx = <Ethereum as Network>::TransactionRequest::default()
    .to(CONTRACT_ADDRESS.parse().unwrap())
    .input(abi.function("decreaseCount")
        .unwrap()
        .encode_input(&[])
        .unwrap()
        .into());

    let result = provider.send_transaction(tx).await;

    match result {
        Ok(pending_tx) => {
            let hash = pending_tx.tx_hash().to_string();
            store_transaction_hash(hash.clone());
            Ok(format!("Decreased count. Transaction hash: {}", hash))
        },
        Err(e) => Err(format!("Failed to decrease count: {:?}", e))
    }
}

#[ic_cdk::update]
async fn get_stored_transaction_hashes() -> Vec<String> {
    get_transaction_hashes()
}

#[ic_cdk::update]
async fn check_balance() -> Result<String, String> {
    let signer = create_icp_signer().await;
    let config = IcpConfig::new(get_rpc_service());
    let provider = ProviderBuilder::new().on_icp(config);
    
    match provider.get_balance(signer.address()).await {
        Ok(balance) => Ok(balance.to_string()),
        Err(e) => Err(format!("Failed to get balance: {:?}", e))
    }
}

// #[ic_cdk::update]
// async fn check_transaction_status(tx_hash: String) -> Result<String, String> {
//     let config = IcpConfig::new(get_rpc_service());
//     let provider = ProviderBuilder::new().on_icp(config);
    
//     match provider.get_transaction_receipt(&tx_hash.parse().unwrap()).await {
//         Ok(Some(receipt)) => {
//             Ok(format!("Transaction status: {:?}", receipt.status))
//         },
//         Ok(None) => {
//             Ok("Transaction pending".to_string())
//         },
//         Err(e) => {
//             Err(format!("Failed to get transaction status: {:?}", e))
//         }
//     }
// }

ic_cdk::export_candid!();  
