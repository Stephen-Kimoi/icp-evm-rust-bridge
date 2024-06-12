mod evm_rpc;
mod eth_call;
use eth_call::call_smart_contract;
use ethers_core::abi::Token;
use evm_rpc::{
    Block, BlockTag, EthMainnetService, EvmRpcCanister, GetBlockByNumberResult,
    MultiGetBlockByNumberResult, RpcServices,
};
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
    EcdsaPublicKeyResponse, SignWithEcdsaArgument, SignWithEcdsaResponse,
};

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

#[ic_cdk::update]
async fn call_increase_count() -> u64 {
    // let contract_address = "0xAed5d7b083ad30ad6B50f698427aD4907845AAc3".to_string();

    let abi_json = r#"
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

    let abi = serde_json::from_str::<ethers_core::abi::Contract>(abi_json).expect("Should serialize"); 
    
    // Call the increaseCount function
    let _: Vec<Token> = call_smart_contract(
        "0xAed5d7b083ad30ad6B50f698427aD4907845AAc3".to_string(), 
        &abi, 
        "increaseCount", 
        &[], 
        "latest",
    ).await; 

    // Call the getCount function to retrieve the updated count
    let count: Vec<Token> = call_smart_contract(
        "0xAed5d7b083ad30ad6B50f698427aD4907845AAc3".to_string(),
        &abi,
        "getCount",
        &[],
        "latest",
    ).await;

    // Extract the count value from the returned tokens 
    let count_value = count
        .get(0)
        .expect("Expected a single value in the return value")
        .clone() 
        .into_uint()
        .expect("Expected a uint256 value"); 
    
    count_value.low_u64()

}

fn key_id() -> EcdsaKeyId {
    EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "dfx_test_key".to_string(), // use EcdsaKeyId::default() for mainnet
    }
}

ic_cdk::export_candid!();  
