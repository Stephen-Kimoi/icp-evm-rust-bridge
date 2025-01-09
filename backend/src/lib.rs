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

    // Create the transaction request
    let mut request = <Ethereum as Network>::TransactionRequest::default()
        .to(CONTRACT_ADDRESS.parse().unwrap())
        .input(abi.function("increaseCount")
            .unwrap()
            .encode_input(&[])
            .unwrap()
            .into())
        .nonce(nonce);
    
    request.set_gas_limit(100_000);

    // Convert to a legacy transaction type that implements SignableTransaction
    let mut tx = TxLegacy {
        nonce: request.nonce.unwrap_or_default(),
        gas_price: request.gas_price.unwrap_or_default(),
        gas_limit: request.gas.unwrap_or_default().try_into().unwrap(),
        to: request.to.unwrap_or_default(),
        value: request.value.unwrap_or_default(),
        input: request.input.data.unwrap_or_default(),
        chain_id: Some(11155111_u64),
    };    
    
    // Sign the transaction
    let signature = signer.sign_transaction(&mut tx)
        .await
        .map_err(|e| format!("Failed to sign transaction: {:?}", e))?;

    // Get the transaction hash
    let hash = tx.signature_hash();

   // Create a signed transaction
    let signed_tx = Signed::new_unchecked(tx, signature, hash);

    // Get the inner transaction from Signed
    let tx_for_sending = signed_tx.tx();

    // Encode the full signed transaction
    let mut encoded_tx = Vec::new();
    tx_for_sending.encode(&mut encoded_tx); 

    // Send the raw transaction
    let result = provider.send_raw_transaction(&encoded_tx).await;

    match result {
        Ok(pending_tx) => {
            let hash = pending_tx.tx_hash().to_string();
            store_transaction_hash(hash.clone());
            Ok(format!("Increased count. Transaction hash: {}", hash))
        },
        Err(e) => {
            Err(format!("Failed to increase count: {:?}", e))
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

ic_cdk::export_candid!();  
