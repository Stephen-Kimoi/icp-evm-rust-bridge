use ethers_core::{abi::{Contract, Token}, types::U64, utils::keccak256, types::Signature as EthSignature}; 
use k256::ecdsa::{RecoveryId, VerifyingKey, Signature }; 
// use ic_cdk::api::call::{call_with_payment, CallResult};
use serde::{Deserialize, Serialize}; 
use crate::evm_rpc::{
    EthSepoliaService, EvmRpcCanister, MultiSendRawTransactionResult, RequestResult, RpcService, RpcServices,
    EVM_RPC, SendRawTransactionResult
}; 
use hex::FromHexError; 
use ethers_core::types::{U256, Bytes}; 
use ic_cdk::api::management_canister::ecdsa::{
    EcdsaPublicKeyResponse, EcdsaPublicKeyArgument, ecdsa_public_key, EcdsaCurve, EcdsaKeyId, sign_with_ecdsa, SignWithEcdsaResponse, 
    SignWithEcdsaArgument
}; 
use sha2::{Sha256, Digest};

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

pub async fn call_smart_contract(
    contract_address: String,
    abi: &Contract,
    function_name: &str,
    args: &[Token],
    is_write_operation: bool,
    chain_id: Option<U64>,
    to: Option<String>,
    gas: Option<U256>,
    value: Option<U256>,
    nonce: Option<U256>,
    max_priority_fee_per_gas: Option<U256>,
    max_fee_per_gas: Option<U256>,
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
            // Handle write operations (send transaction)
            // let caller = ic_cdk::caller();
            let chain_id = chain_id.ok_or("Chain ID is required")?;
            let to = to.ok_or("Destination address is required")?;
            let gas = gas.ok_or("Gas limit is required")?;
            let value = value.unwrap_or_default();
            let nonce = nonce.ok_or("Nonce is required")?;
            let max_priority_fee_per_gas = max_priority_fee_per_gas.ok_or("Max priority fee per gas is required")?;
            let max_fee_per_gas = max_fee_per_gas.ok_or("Max fee per gas is required")?;
    
            // Sign the transaction
            let signed_tx = sign_transaction(
                chain_id,
                to,
                gas,
                value,
                nonce,
                max_priority_fee_per_gas,
                max_fee_per_gas,
                data.to_vec(),
            )
            .await?;
    
            // Send the signed raw transaction
            let network = "EthSepolia".to_string();
            let tx_status = send_raw_transaction(network, signed_tx).await?;
    
            // Handle the transaction status
            let tx_result = match tx_status {
                SendRawTransactionResult::Ok(tx_hash) => {
                    // Transaction was successful
                    // You can optionally return the transaction hash if needed
                    // Ok(vec![Token::String(tx_hash)])
                    Ok(Vec::new())
                }
                SendRawTransactionResult::Err(err) => {
                    Err(format!("Transaction failed: {:?}", err))
                }
            };

            return tx_result;


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
            let cycles = 10_000_000_000; 

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
    nonce: U256,
    max_priority_fee_per_gas: U256,
    max_fee_per_gas: U256,
    data: Vec<u8>,
) -> Result<String, String> {
    use ethers_core::types::transaction::eip1559::Eip1559TransactionRequest;
    use ethers_core::types::Signature;
    use ethers_core::types::H160;
    use std::str::FromStr;
    const EIP1559_TX_ID: u8 = 2;

    let data = Some(Bytes::from(data));
    let tx = Eip1559TransactionRequest {
        chain_id: Some(chain_id),
        from: None,
        to: Some(
            H160::from_str(&to)
                .map_err(|err| format!("Failed to parse the destination address: {}", err))?
                .into(),
        ),
        gas: Some(gas),
        value: Some(value),
        nonce: Some(nonce),
        data,
        access_list: Default::default(),
        max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
        max_fee_per_gas: Some(max_fee_per_gas),
    };

    let mut unsigned_tx_bytes = tx.rlp().to_vec();
    unsigned_tx_bytes.insert(0, EIP1559_TX_ID);
    let txhash = keccak256(&unsigned_tx_bytes);
    let (pubkey, signature) = pubkey_and_signature(txhash.to_vec()).await;
    let signature = Signature {
        v: y_parity(&txhash, &signature, &pubkey),
        r: U256::from_big_endian(&signature.signature[0..32]),
        s: U256::from_big_endian(&signature.signature[32..64]),
    };
    let mut signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
    signed_tx_bytes.insert(0, EIP1559_TX_ID);
    Ok(format!("0x{}", hex::encode(&signed_tx_bytes)))
}

async fn send_raw_transaction(network: String, raw_tx: String) -> Result<SendRawTransactionResult, String> {
    let config = None;
    let services = match network.as_str() {
        "EthSepolia" => RpcServices::EthSepolia(Some(vec![EthSepoliaService::Alchemy])),
        "EthMainnet" => RpcServices::EthMainnet(None),
        _ => RpcServices::EthSepolia(None),
    };
    let cycles = 10000000;
    match EvmRpcCanister::eth_sendRawTransaction(services, config, raw_tx, cycles).await {
        Ok((res,)) => match res {
            MultiSendRawTransactionResult::Consistent(result) => Ok(result),
            MultiSendRawTransactionResult::Inconsistent(_) => Err("Status is inconsistent".to_string()),
        },
        Err(e) => Err(format!("Error: {:?}", e)),
    }
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
        name: "test_key_1".to_string(), // use EcdsaKeyId::default() for mainnet
    }
}

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


fn y_parity(txhash: &[u8; 32], signature: &SignWithEcdsaResponse, pubkey: &EcdsaPublicKeyResponse) -> u64 {
    let r = U256::from_big_endian(&signature.signature[0..32]);
    let s = U256::from_big_endian(&signature.signature[32..64]);
    let eth_sig = EthSignature { r, s, v: 0 };

    let msg = Sha256::digest(txhash);
    let vkey = VerifyingKey::from_sec1_bytes(&pubkey.public_key).expect("Invalid public key");

    let mut sig_bytes = [0u8; 64];
    eth_sig.r.to_big_endian(&mut sig_bytes[0..32]);
    eth_sig.s.to_big_endian(&mut sig_bytes[32..64]);

    let signature = Signature::from_bytes(&sig_bytes.into()).expect("Invalid signature");

    for i in 0..2 {
        let rid = RecoveryId::from_byte(i).expect("Invalid recovery ID");
        if let Ok(recovered_key) = VerifyingKey::recover_from_prehash(
            msg.as_slice(),
            &signature,
            rid,
        ) {
            if vkey == recovered_key {
                return i as u64 + 27;
            }
        }
    }

    panic!("Failed to determine y-parity");
}