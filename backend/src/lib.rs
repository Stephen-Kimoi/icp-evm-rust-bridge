mod eth_call;
mod store_transactions;
use candid::Principal;
use eth_call::{ call_smart_contract, get_ecdsa_public_key };
use store_transactions::{store_transaction_hash, get_transaction_hashes}; 
use ethers_core::{abi::Address, k256::elliptic_curve::{sec1::ToEncodedPoint, PublicKey}, types::U256, utils::keccak256};
use evm_rpc_canister_types::{
    EvmRpcCanister, GetTransactionCountArgs, 
    RpcServices, Block, 
    EthMainnetService, MultiGetBlockByNumberResult, GetBlockByNumberResult
};
use k256::Secp256k1;
use evm_rpc_canister_types::BlockTag;

pub const EVM_RPC_CANISTER_ID: Principal =
    Principal::from_slice(b"\x00\x00\x00\x00\x02\x30\x00\xCC\x01\x01"); // 7hfb6-caaaa-aaaar-qadga-cai
pub const EVM_RPC: EvmRpcCanister = EvmRpcCanister(EVM_RPC_CANISTER_ID);

#[ic_cdk::update]
async fn get_latest_ethereum_block() -> Block {
    let rpc_providers = RpcServices::EthMainnet(Some(vec![EthMainnetService::Cloudflare]));

    let cycles = 10_000_000_000;
    let (result,) =
        EvmRpcCanister::eth_get_block_by_number(
            &EVM_RPC,
            rpc_providers, 
            None, 
            BlockTag::Latest, 
            cycles
        )
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

// Generating an ethereum address for the canister
#[ic_cdk::update] 
pub async fn get_canister_eth_address() -> String {
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

    let result = call_smart_contract(
        CONTRACT_ADDRESS.to_string(), 
        &abi,
        "increaseCount",
        &[],
        true,
        Some(U256::from(11155111)), // Sepolia chain ID as U256
    )
    .await;

    match result {
        Ok(tx_hash_tokens) => {
            // Convert Vec<Token> to String
            let tx_hash = tx_hash_tokens
                .get(0)
                .ok_or("Expected a single value in the return value")?
                .clone()
                .into_string()
                .ok_or("Expected a string value")?;

            // Extract the actual hash from the string
            let tx_hash_cleaned = tx_hash
                .trim_start_matches("Ok(Some(\"")
                .trim_end_matches("\"))");

            ic_cdk::println!("Transaction sent successfully. Hash: {:?}", tx_hash_cleaned);

            // Store the transaction hash
            store_transaction_hash(tx_hash_cleaned.to_string());
            Ok(format!("Increased count. Transaction hash: {:?}", tx_hash_cleaned))
        },
        Err(e) => {
            ic_cdk::println!("Error sending transaction: {:?}", e);
            Err(format!("Failed to send transaction: {:?}", e))
        }
    }
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
        None, // Chain ID
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

    call_smart_contract(
        CONTRACT_ADDRESS.to_string(),
        &abi,
        "decreaseCount",
        &[],
        true, // This is a write operation
        None
    )
    .await?;

    Ok("Decreased count".to_string())
} 

#[ic_cdk::update]
async fn get_stored_transaction_hashes() -> Vec<String> {
    get_transaction_hashes()
}

ic_cdk::export_candid!();  
