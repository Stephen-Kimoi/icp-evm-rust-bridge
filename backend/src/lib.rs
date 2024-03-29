#![allow(non_snake_case)]

mod evm_rpc;
use evm_rpc::{
    Block, BlockTag, EthMainnetService, EvmRpcCanister, GetBlockByNumberResult,
    MultiGetBlockByNumberResult, RpcApi, RpcServices,
};

#[ic_cdk::update]
async fn getLatestEthereumBlock() -> Block {
    let cycles = 10_000_000_000;
    let rpc_providers = RpcServices::EthMainnet(Some(vec![EthMainnetService::Cloudflare]));

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
