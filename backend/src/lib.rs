#![allow(non_snake_case)]

mod evm_rpc;
use candid::Principal;
use evm_rpc::{
    eth_get_block_by_number, Block, BlockTag, GetBlockByNumberResult, MultiGetBlockByNumberResult,
    RpcServices,
};

const EVM_RPC: Principal = Principal::from_slice(b"00000000023000CC0101"); // 7hfb6-caaaa-aaaar-qadga-cai

#[ic_cdk::update]
async fn getLatestEthereumBlock() -> Block {
    let (result,) = eth_get_block_by_number(
        EVM_RPC,
        RpcServices::EthMainnet(None),
        None,
        BlockTag::Latest,
        81453120000,
    )
    .await
    .unwrap();

    match result {
        MultiGetBlockByNumberResult::Consistent(r) => match r {
            GetBlockByNumberResult::Ok(block) => block,
            GetBlockByNumberResult::Err(err) => panic!("{err:?}"),
        },
        MultiGetBlockByNumberResult::Inconsistent(_) => panic!("Inconsistent result"),
    }
}
