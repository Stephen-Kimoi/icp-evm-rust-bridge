use candid::CandidType;

#[derive(CandidType)]
struct Block {
    baseFeePerGas: u128,
    difficulty: u128,
    extraData: String,
    gasLimit: u128,
    gasUsed: u128,
    hash: String,
    logsBloom: String,
    miner: String,
    mixHash: String,
    nonce: u128,
    number: u128,
    parentHash: String,
    receiptsRoot: String,
    sha3Uncles: String,
    size: u128,
    stateRoot: String,
    timestamp: u128,
    totalDifficulty: u128,
    transactions: Vec<String>,
    transactionsRoot: Option<String>,
    uncles: Vec<String>,
}

#[ic_cdk::query]
fn getLatestEthereumBlock() -> Block {
    Block {
        baseFeePerGas: 420,
        difficulty: 12,
        extraData: String::new(),
        gasLimit: 123,
        gasUsed: 312,
        hash: String::new(),
        logsBloom: String::new(),
        miner: String::new(),
        mixHash: String::new(),
        nonce: 23,
        number: 23,
        parentHash: String::new(),
        receiptsRoot: String::new(),
        sha3Uncles: String::new(),
        size: 23,
        stateRoot: String::new(),
        timestamp: 41,
        totalDifficulty: 12,
        transactions: vec![],
        transactionsRoot: None,
        uncles: vec![],
    }
}
