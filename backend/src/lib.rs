#![allow(non_snake_case)]

mod evm_rpc;
use evm_rpc::{
    Block, BlockTag, EvmRpcCanister, GetBlockByNumberResult, MultiGetBlockByNumberResult,
    RpcServices,
};

#[ic_cdk::update]
async fn getLatestEthereumBlock() -> Block {
    let cycles = 10_000_000_000;
    let (result,) = EvmRpcCanister::eth_get_block_by_number(
        RpcServices::EthMainnet(None),
        None,
        BlockTag::Latest,
        cycles,
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
