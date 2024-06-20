mod evm_rpc;
mod eth_call;
use eth_call::{ call_smart_contract, get_ecdsa_public_key };
use ethers_core::{abi::Address, k256::elliptic_curve::{sec1::ToEncodedPoint, PublicKey}, utils::keccak256};
use evm_rpc::{
    Block, BlockTag, EthMainnetService, EvmRpcCanister, GetBlockByNumberResult,
    MultiGetBlockByNumberResult, RpcServices, EthSepoliaService
};
use k256::Secp256k1;
use ethers_core::types::{U64, U256};

#[ic_cdk::update]
async fn get_latest_ethereum_block() -> Block {
    let rpc_providers = RpcServices::EthMainnet(Some(vec![EthMainnetService::Cloudflare]));

    let cycles = 10_000_000_000;
    let (result,) =
        EvmRpcCanister::eth_get_block_by_number(rpc_providers, None, BlockTag::Latest, cycles)
            .await
            .expect("Call failed");

    match result {
        MultiGetBlockByNumberResult::Consistent(r) => match r {
            GetBlockByNumberResult::Ok(block) => block,
            GetBlockByNumberResult::Err(err) => panic!("{err:?}"),
        },
        MultiGetBlockByNumberResult::Inconsistent(_) => {
            panic!("RPC providers gave inconsistent results")
        }
    }
}

#[ic_cdk::update] 
async fn get_canister_eth_address() -> String {
    let res = get_ecdsa_public_key().await; 
    let pubkey = res.public_key; 

    let key: PublicKey<Secp256k1> = PublicKey::from_sec1_bytes(&pubkey)
        .expect("Failed to pass the public key as SEC1"); 
    let point = key.to_encoded_point(false); 
    let point_bytes = point.as_bytes(); 
    assert_eq!(point_bytes[0], 0x04); 
    let hash = keccak256(&point_bytes[1..]); 
    let self_address = ethers_core::utils::to_checksum(&Address::from_slice(&hash[12..32]), None); 

    self_address
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

#[ic_cdk::update]
async fn call_increase_count() -> Result<String, String> {
    let abi = get_abi();

    // Get the canister's Ethereum address
    let canister_address = get_canister_eth_address().await;
    
    // Get the current nonce
    let nonce = get_nonce(&canister_address).await?;

    let result = call_smart_contract(
        CONTRACT_ADDRESS.to_string(),
        &abi,
        "increaseCount",
        &[],
        true, // This is a write operation
        Some(U64::from(11155111)), // Chain ID for Sepolia testnet
        Some(CONTRACT_ADDRESS.to_string()),
        Some(U256::from(100000)), // Gas limit
        Some(U256::from(0)), // Value (0 ETH)
        Some(nonce), // Nonce included here
        Some(U256::from(1_500_000_000u64)), // Max priority fee per gas (1.5 Gwei)
        Some(U256::from(20_000_000_000u64)), // Max fee per gas (20 Gwei)
    )
    .await?;

    Ok("Increased count".to_string())
}

#[ic_cdk::update]
async fn get_count() -> Result<u64, String> {
    let abi = get_abi();
    
    let result = call_smart_contract(
        CONTRACT_ADDRESS.to_string(), 
        &abi, 
        "getCount", 
        &[], 
        false, // This is a read operation
        None, None, None, None, None, None, None // These parameters are not needed for read operations
    ).await?;

    let count_value = result
        .get(0)
        .ok_or("Expected a single value in the return value")?
        .clone()
        .into_uint()
        .ok_or("Expected a uint256 value")?;

    Ok(count_value.low_u64())
}

#[ic_cdk::update]
async fn call_decrease_count() -> Result<String, String> {
    let abi = get_abi();

    // Get the canister's Ethereum address
    let canister_address = get_canister_eth_address().await;
    
    // Get the current nonce
    let nonce = get_nonce(&canister_address).await?;

    let result = call_smart_contract(
        CONTRACT_ADDRESS.to_string(),
        &abi,
        "decreaseCount",
        &[],
        true, // This is a write operation
        Some(U64::from(11155111)), // Chain ID for Sepolia testnet
        Some(CONTRACT_ADDRESS.to_string()),
        Some(U256::from(100000)), // Gas limit
        Some(U256::from(0)), // Value (0 ETH)
        None, // Nonce (will be fetched automatically)
        Some(U256::from(1_500_000_000u64)), // Max priority fee per gas (1.5 Gwei)
        Some(U256::from(20_000_000_000u64)), // Max fee per gas (20 Gwei)
    )
    .await?;

    Ok("Decreased count".to_string())
}

async fn get_nonce(address: &str) -> Result<U256, String> {
    let rpc_services = RpcServices::EthSepolia(Some(vec![EthSepoliaService::Alchemy]));
    let config = None;
    let cycles = 10_000_000_000;

    match EvmRpcCanister::eth_get_transaction_count(rpc_services, config, address.to_string(), "latest".to_string(), cycles).await {
        Ok((count,)) => Ok(count),
        Err(e) => Err(format!("Failed to get nonce: {:?}", e)),
    }
}

ic_cdk::export_candid!();  
