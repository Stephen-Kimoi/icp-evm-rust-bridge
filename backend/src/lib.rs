#![allow(non_snake_case)]

mod evm_rpc;
use evm_rpc::{
    eth_get_block_by_number, Block, BlockTag, GetBlockByNumberResult, MultiGetBlockByNumberResult,
    RpcServices, CANISTER_ID
};

#[ic_cdk::update]
async fn getLatestEthereumBlock() -> Block {
    let (result,) = eth_get_block_by_number(
        CANISTER_ID,
        RpcServices::EthMainnet(None),
        None,
        BlockTag::Latest,
        81453120000,
    )
    .await
    .expect("Call failed");

    match result {
        MultiGetBlockByNumberResult::Consistent(r) => match r {
            GetBlockByNumberResult::Ok(block) => block,
            GetBlockByNumberResult::Err(err) => panic!("{err:?}"),
        },
        MultiGetBlockByNumberResult::Inconsistent(_) => panic!("Inconsistent result"),
    }
}
