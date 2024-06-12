use ethers_core::abi::{Token, Contract}; 
// use ic_cdk::api::call::{call_with_payment, CallResult};
use serde::{Deserialize, Serialize}; 
use crate::evm_rpc::{EthSepoliaService, RequestResult, RpcService, EVM_RPC}; 
use hex::FromHexError; 

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
   block_number: &str
) -> Vec<Token> {
    let f = match abi.functions_by_name(function_name).map(|v| &v[..]) {
        Ok([f]) => f,
        Ok(fs) => panic!(
            "Found {} function overloads. Please pass one of the following: {}",
            fs.len(),
            fs.iter()
                .map(|f| format!("{:?}", f.signature())) // Changed this from (abi_signature)
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
    let json_rpc_payload = serde_json::to_string(&JsonRpcRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "eth_call".to_string(),
        params: (
            EthCallParams {
                to: contract_address,
                data: to_hex(&data),
            },
            block_number.to_string(),
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
            f.decode_output(&result).expect("Error decoding output")
        }
        RequestResult::Err(err) => panic!("Response error: {err:?}"),
    }
}

// HELPER FUNCTIONS

fn to_hex(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

fn from_hex(data: &str) -> Result<Vec<u8>, FromHexError> {
    hex::decode(&data[2..])
}