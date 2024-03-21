#![allow(non_snake_case)]

mod evm_rpc;
use evm_rpc::{
    Block, BlockTag, EthMainnetService, EvmRpcCanister, GetBlockByNumberResult,
    MultiGetBlockByNumberResult, RpcServices,
};

#[ic_cdk::update]
async fn getLatestEthereumBlock() -> Block {
    let cycles = 10_000_000_000;
    let rpc_providers = vec![EthMainnetService::Cloudflare];
    let (result,) = EvmRpcCanister::eth_get_block_by_number(
        RpcServices::EthMainnet(Some(rpc_providers)),
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
