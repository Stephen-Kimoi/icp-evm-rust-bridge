mod evm_rpc;
mod eth_call;
use eth_call::call_smart_contract;
use ethers_core::{abi::{Address, Token}, k256::elliptic_curve::{sec1::ToEncodedPoint, PublicKey}, utils::keccak256};
use evm_rpc::{
    Block, BlockTag, EthMainnetService, EvmRpcCanister, GetBlockByNumberResult,
    MultiGetBlockByNumberResult, RpcServices,
};
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
    EcdsaPublicKeyResponse, SignWithEcdsaArgument, SignWithEcdsaResponse,
};
use k256::Secp256k1;

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
async fn get_ecdsa_public_key() -> EcdsaPublicKeyResponse {
    let (pub_key,) = ecdsa_public_key(EcdsaPublicKeyArgument {
        key_id: key_id(),
        ..Default::default()
    })
    .await
    .expect("Failed to get public key");
    pub_key
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

#[ic_cdk::update]
async fn sign_hash_with_ecdsa(message_hash: Vec<u8>) -> SignWithEcdsaResponse {
    let (signature,) = sign_with_ecdsa(SignWithEcdsaArgument {
        message_hash,
        key_id: key_id(),
        ..Default::default()
    })
    .await
    .expect("Failed to sign");
    signature
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
async fn call_increase_count() -> String {
    let abi = get_abi();
    call_smart_contract(
        CONTRACT_ADDRESS.to_string(),
        &abi,
        "increaseCount",
        &[],
        "latest",
    )
    .await; 

    return "Increased count".to_string()
}

#[ic_cdk::update]
async fn get_count() -> u64 {
    // Get the ABI
    let abi = get_abi();
    
    // Call the getCount function
    let count: Vec<Token> = call_smart_contract(
        CONTRACT_ADDRESS.to_string(), 
        &abi, 
        "getCount", 
        &[], 
        "latest", 
    ).await; 

    let count_value = count
        .get(0)
        .expect("Expected a single value in the return value")
        .clone() 
        .into_uint()
        .expect("Expected a uint256 value"); 

    count_value.low_u64()
}

#[ic_cdk::update]
async fn call_decrease_count() -> String {
    // Get the ABI
    let abi = get_abi();

    // Call the decreaseCount function
    call_smart_contract(
        CONTRACT_ADDRESS.to_string(), 
        &abi, 
        "decreaseCount", 
        &[], 
        "latest", 
    ).await; 
    // .map_err(|err| format!("Error calling decreaseCount: {err:?}"))?; 

    return "Decreased count".to_string();
}

fn key_id() -> EcdsaKeyId {
    EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "dfx_test_key".to_string(), // use EcdsaKeyId::default() for mainnet
    }
}

ic_cdk::export_candid!();  
