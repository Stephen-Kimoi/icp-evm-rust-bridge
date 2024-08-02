use ethers_core::{abi::{Contract, Token}, types::{U64, U256, Bytes, H160, transaction::eip1559::Eip1559TransactionRequest }, utils::keccak256};
use serde::{Deserialize, Serialize};
use evm_rpc_canister_types::{
    EvmRpcCanister, GetTransactionCountResult,
    MultiGetTransactionCountResult, EthSepoliaService, MultiSendRawTransactionResult, RpcServices, SendRawTransactionResult, RpcService, RequestResult, RpcConfig
};
use hex::FromHexError;
use ic_cdk::api::management_canister::ecdsa::{
    EcdsaPublicKeyResponse, EcdsaPublicKeyArgument, ecdsa_public_key, EcdsaCurve, EcdsaKeyId, sign_with_ecdsa, SignWithEcdsaResponse, 
    SignWithEcdsaArgument
};
use ic_cdk::api::call::{CallResult, call_with_payment}; 
use std::str::FromStr;
use candid::{Nat, Principal}; 
use crate::get_canister_eth_address; 
use evm_rpc_canister_types::BlockTag;

pub const EVM_RPC_CANISTER_ID: Principal =
    Principal::from_slice(b"\x00\x00\x00\x00\x02\x30\x00\xCC\x01\x01"); // 7hfb6-caaaa-aaaar-qadga-cai
pub const EVM_RPC: EvmRpcCanister = EvmRpcCanister(EVM_RPC_CANISTER_ID);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EthCallParams {
    pub to: String,
    pub data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub id: u64,
    pub jsonrpc: String,
    pub method: String,
    pub params: (EthCallParams, String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcResult {
    result: Option<String>,
    error: Option<JsonRpcError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    code: isize,
    message: String,
}

// const CHAIN_ID: u128 = 1337;
const CHAIN_ID: u128 = 11155111; // Sepolia
const GAS: u128 = 300_000;
const MAX_FEE_PER_GAS: u128 = 156_083_066_522_u128;
const MAX_PRIORITY_FEE_PER_GAS: u128 = 3_000_000_000;

pub async fn call_smart_contract(
    contract_address: String,
    abi: &Contract,
    function_name: &str,
    args: &[Token],
    is_write_operation: bool,
    value: Option<U256>,
) -> Result<Vec<Token>, String> {

    let f = match abi.functions_by_name(function_name).map(|v| &v[..]) {
        Ok([f]) => f,
        Ok(fs) => panic!(
            "Found {} function overloads. Please pass one of the following: {}",
            fs.len(),
            fs.iter()
                .map(|f| format!("{:?}", f.signature()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Err(_) => abi
            .functions()
            .find(|f| function_name == f.signature())
            .expect("Function not found"),
    };

    let data = f
        .encode_input(args)
        .expect("Error while encoding input args");

    if is_write_operation {
        let value = value.unwrap_or_default();

        let signed_tx = sign_transaction(
            U64::from(CHAIN_ID as u64),
            contract_address,
            U256::from(GAS),
            value,
            next_id().await, 
            U256::from(MAX_PRIORITY_FEE_PER_GAS),
            U256::from(MAX_FEE_PER_GAS),
            data.to_vec(),
        )
        .await?;

        let result = EVM_RPC
            .eth_send_raw_transaction(
                RpcServices::EthSepolia(Some(vec![
                    EthSepoliaService::PublicNode,
                    EthSepoliaService::BlockPi,
                    EthSepoliaService::Ankr,
                ])),
                None::<evm_rpc_canister_types::RpcConfig>,
                signed_tx.clone(),
                10_000_000_000,
            ).await 
            .map_err(|e| format!("Failed to call eth_sendRawTransaction: {:?}", e))?;

            match result {
                (MultiSendRawTransactionResult::Consistent(send_result),) => {
                    match send_result {
                        SendRawTransactionResult::Ok(tx_status) => {
                            // Convert SendRawTransactionStatus to String
                            Ok(vec![Token::String(format!("{:?}", tx_status))])
                        },
                        SendRawTransactionResult::Err(err) => Err(format!("Transaction failed: {:?}", err)),
                    }
                }
                (MultiSendRawTransactionResult::Inconsistent(results),) => {
                    let errors: Vec<String> = results
                        .into_iter()
                        .map(|(service, send_result)| match send_result {
                            SendRawTransactionResult::Ok(tx_status) => format!("Success with status: {:?}", tx_status),
                            SendRawTransactionResult::Err(err) => format!("Service {:?} failed: {:?}", service, err),
                        })
                        .collect();
                    Err(format!("Inconsistent results: {:?}", errors))
                }
            }

    } else {
        let json_rpc_payload = serde_json::to_string(&JsonRpcRequest {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "eth_call".to_string(),
            params: (
                EthCallParams {
                    to: contract_address,
                    data: to_hex(&data),
                },
                "latest".to_string(),
            ),
        })
        .expect("Error while encoding JSON-RPC request");

        let rpc_provider = RpcService::EthSepolia(EthSepoliaService::BlockPi); 
        let max_response_bytes = 2048;
        let cycles = 10_000_000_000; // Increase the cycles here

        let res = match EVM_RPC
            .request(rpc_provider, json_rpc_payload, max_response_bytes, cycles)
            .await
        {
            Ok((res,)) => res,
            Err(e) => ic_cdk::trap(format!("Error: {:?}", e).as_str()),
        };
         
        match res {
            RequestResult::Ok(ok) => {
                let json: JsonRpcResult =
                    serde_json::from_str(&ok).expect("JSON was not well-formatted");
                let result = from_hex(&json.result.expect("Unexpected JSON response")).unwrap();
                Ok(f.decode_output(&result).expect("Error decoding output"))
            }
            RequestResult::Err(err) => Err(format!("Response error: {err:?}")),
        }

    }
}


async fn sign_transaction(
    chain_id: U64,
    to: String,
    gas: U256,
    value: U256,
    nonce: Nat,
    max_priority_fee_per_gas: U256,
    max_fee_per_gas: U256,
    data: Vec<u8>,
) -> Result<String, String> {
    const EIP1559_TX_ID: u8 = 2;

    let tx = Eip1559TransactionRequest {
        chain_id: Some(chain_id),
        from: None,
        to: Some(H160::from_str(&to).map_err(|e| format!("Invalid 'to' address: {}", e))?.into()),
        gas: Some(gas),
        value: Some(value), 
        nonce: Some(nat_to_u256(&nonce)),
        data: Some(Bytes::from(data)),
        access_list: Default::default(),
        max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
        max_fee_per_gas: Some(max_fee_per_gas),
    };

    let mut unsigned_tx_bytes = tx.rlp().to_vec();
    unsigned_tx_bytes.insert(0, EIP1559_TX_ID);

    let txhash = keccak256(&unsigned_tx_bytes);

    let (pubkey, signature) = pubkey_and_signature(txhash.to_vec()).await;

    let y_parity = y_parity(&txhash, &signature.signature, &pubkey.public_key);
    let signature = ethers_core::types::Signature {
        r: U256::from_big_endian(&signature.signature[0..32]),
        s: U256::from_big_endian(&signature.signature[32..64]),
        v: y_parity as u64,
    };

    let mut signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
    signed_tx_bytes.insert(0, EIP1559_TX_ID);

    Ok(format!("0x{}", hex::encode(&signed_tx_bytes)))
}


// HELPER FUNCTIONS
fn to_hex(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

fn from_hex(data: &str) -> Result<Vec<u8>, FromHexError> {
    hex::decode(&data[2..])
}

fn key_id() -> EcdsaKeyId {
    EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "test_key_1".to_string(), // use EcdsaKeyId::default() for mainnet use test_key_1 for testnet
    }
}

fn nat_to_u256(n: &Nat) -> U256 {
    let be_bytes = n.0.to_bytes_be();
    U256::from_big_endian(&be_bytes)
}

// Getting ECDSA public key associated to your canister
pub async fn get_ecdsa_public_key() -> EcdsaPublicKeyResponse {
    let (pub_key,) = ecdsa_public_key(EcdsaPublicKeyArgument {
        key_id: key_id(),
        ..Default::default()
    })
    .await
    .expect("Failed to get public key");
    pub_key
}

// Implementation of pubkey_and_signature function
async fn pubkey_and_signature(txhash: Vec<u8>) -> (EcdsaPublicKeyResponse, SignWithEcdsaResponse) {
    // Get the public key
    let public_key = get_ecdsa_public_key().await;

    // Generate the signature
    let (signature,) = sign_with_ecdsa(SignWithEcdsaArgument {
        message_hash: txhash,
        key_id: key_id(),
        ..Default::default()
    })
    .await
    .expect("Failed to generate signature");

    (public_key, signature)
}


fn y_parity(prehash: &[u8], sig: &[u8], pubkey: &[u8]) -> u64 {
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

    let orig_key = VerifyingKey::from_sec1_bytes(pubkey).expect("failed to parse the pubkey");
    let signature = Signature::try_from(sig).unwrap();
    for parity in [0u8, 1] {
        let recid = RecoveryId::try_from(parity).unwrap();
        let recovered_key = VerifyingKey::recover_from_prehash(prehash, &signature, recid)
            .expect("failed to recover key");
        if recovered_key == orig_key {
            return parity as u64;
        }
    }

    panic!(
        "failed to recover the parity bit from a signature; sig: {}, pubkey: {}",
        hex::encode(sig),
        hex::encode(pubkey)
    )
}

async fn next_id() -> Nat {
    let res: CallResult<(MultiGetTransactionCountResult,)> = call_with_payment(
        EVM_RPC.0, // Principal
        "eth_getTransactionCount", // Method name
        (
            RpcServices::EthSepolia(Some(vec![EthSepoliaService::BlockPi])),
            None::<RpcConfig>,
            crate::GetTransactionCountArgs {
                address: get_canister_eth_address().await, 
                block: BlockTag::Latest, // Correct BlockTag type
            },
            2_000_000_000, // Cycles
        ),
        2_000_000_000, // Cycles
    )
    .await;

    match res {
        Ok((MultiGetTransactionCountResult::Consistent(GetTransactionCountResult::Ok(id)),)) => id.into(),
        Ok((inconsistent,)) => ic_cdk::trap(&format!("Inconsistent: {:?}", inconsistent)), // Use {:?} for Debug formatting
        Err(err) => ic_cdk::trap(&format!("{:?}", err)),
    }
}
