// mod evm_rpc;
mod eth_call;
use candid::{Nat, Principal};
use eth_call::{ call_smart_contract, get_ecdsa_public_key };
use ethers_core::{abi::Address, k256::elliptic_curve::{sec1::ToEncodedPoint, PublicKey}, utils::keccak256};
// use evm_rpc::{
//     Block, BlockTag, EthMainnetService, EvmRpcCanister, GetBlockByNumberResult,
//     MultiGetBlockByNumberResult, RpcServices, EthSepoliaService, GetTransactionCountArgs, GetTransactionCountResult, MultiGetTransactionCountResult
// };
use evm_rpc_canister_types::{
    EvmRpcCanister, GetTransactionCountArgs, GetTransactionCountResult,
    MultiGetTransactionCountResult, EthSepoliaService, MultiSendRawTransactionResult, RpcServices, SendRawTransactionResult, SendRawTransactionStatus, RpcService, RequestResult, RpcConfig, Block, 
    EthMainnetService, MultiGetBlockByNumberResult, GetBlockByNumberResult
};
use k256::Secp256k1;
use ethers_core::types::U256;
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
    let canister_address = get_canister_eth_address().await;
    let nonce = get_nonce(&canister_address).await?;

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
        Ok(tx_hash) => {
            ic_cdk::println!("Transaction sent successfully. Hash: {:?}", tx_hash);
            Ok(format!("Increased count. Transaction hash: {:?}", tx_hash))
        },
        Err(e) => {
            ic_cdk::println!("Error sending transaction: {:?}", e);
            Err(format!("Failed to send transaction: {:?}", e))
        }
    }
}

// #[ic_cdk::update]
// async fn call_increase_count_with_retry() -> Result<String, String> {
//     for attempt in 0..3 {
//         match call_increase_count().await {
//             Ok(result) => return Ok(result),
//             Err(e) if attempt < 2 => {
//                 ic_cdk::println!("Attempt {} failed: {}. Retrying...", attempt + 1, e);
//                 ic_cdk::timer::sleep(std::time::Duration::from_secs(5)).await;
//             }
//             Err(e) => return Err(e),
//         }
//     }
//     Err("Max retries reached".to_string())
// }

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

async fn get_nonce(address: &str) -> Result<U256, String> {
    let rpc_services = RpcServices::EthSepolia(Some(vec![EthSepoliaService::Alchemy]));

    let params = GetTransactionCountArgs {
        address: address.to_string(),
        block: BlockTag::Latest,
    };

    let (result,) = EVM_RPC
      .eth_get_transaction_count(
        rpc_services, 
        None, 
        params.clone(),  
        2_000_000_000_u128
    ).await 
    .unwrap_or_else(|e| {
        panic!(
            "failed to get transaction count for {:?}, error: {:?}",
            params, e
        )
    });

    match result {
        MultiGetTransactionCountResult::Consistent(count_result) => match count_result {
            GetTransactionCountResult::Ok(count) => Ok(nat_to_u256(&count)),
            GetTransactionCountResult::Err(error) => {
                Err(format!("failed to get transaction count for {:?}, error: {:?}", params, error))
            }
        },
        MultiGetTransactionCountResult::Inconsistent(_) => Err("Inconsistent RPC results".to_string()),
    }
}

// Helper function to convert Nat to U256
fn nat_to_u256(n: &Nat) -> U256 {
    let be_bytes = n.0.to_bytes_be();
    U256::from_big_endian(&be_bytes)
}

ic_cdk::export_candid!();  
