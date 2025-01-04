mod store_transactions;
use candid::Principal;
// use eth_call::{ call_smart_contract, get_ecdsa_public_key };
use store_transactions::{store_transaction_hash, get_transaction_hashes}; 
// use ethers_core::{k256::elliptic_curve::{sec1::ToEncodedPoint, PublicKey}, types::U256, utils::keccak256};
use evm_rpc_canister_types::{
    EvmRpcCanister, 
    RpcServices, Block, 
    EthMainnetService, MultiGetBlockByNumberResult, GetBlockByNumberResult
};
// use k256::Secp256k1;
use evm_rpc_canister_types::BlockTag;
use alloy::{
    network::Network, providers::{Provider, ProviderBuilder}, signers::icp::IcpSigner, transports::icp::{IcpConfig, RpcApi, RpcService}
};
use alloy::signers::Signer;
// use ethers_core::types::TransactionRequest;
// use alloy_rpc_types_eth::TransactionRequest;
use alloy::network::Ethereum;
// use alloy::network::TransactionBuilder;

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
    RpcService::Custom(RpcApi {
        url: "https://ic-alloy-evm-rpc-proxy.kristofer-977.workers.dev/eth-sepolia".to_string(),
        headers: None,
    })
}

async fn create_icp_signer() -> IcpSigner {
    let key_name = "dfx_test_key".to_string(); // For local testing
    IcpSigner::new(vec![], &key_name, None).await.unwrap()
}

#[ic_cdk::update]
async fn get_canister_eth_address() -> String {
    let signer = create_icp_signer().await;
    signer.address().to_string()
}

#[ic_cdk::update]
async fn call_increase_count() -> Result<String, String> {
    // let signer = create_icp_signer().await;
    let config = IcpConfig::new(get_rpc_service());
    let provider = ProviderBuilder::new().on_icp(config);
    let abi = get_abi();

    let tx = <Ethereum as Network>::TransactionRequest::default()
    .to(CONTRACT_ADDRESS.parse().unwrap())
    .input(abi.function("increaseCount")
        .unwrap()
        .encode_input(&[])
        .unwrap()
        .into());

    let result = provider.send_transaction(tx).await;

    match result {
        Ok(pending_tx) => {
            let hash = pending_tx.tx_hash().to_string();
            store_transaction_hash(hash.clone());
            Ok(format!("Increased count. Transaction hash: {}", hash))
        },
        Err(e) => Err(format!("Failed to increase count: {:?}", e))
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
