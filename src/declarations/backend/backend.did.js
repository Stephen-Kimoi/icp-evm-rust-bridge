export const idlFactory = ({ IDL }) => {
  const Result = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text });
  const Block = IDL.Record({
    'miner' : IDL.Text,
    'totalDifficulty' : IDL.Nat,
    'receiptsRoot' : IDL.Text,
    'stateRoot' : IDL.Text,
    'hash' : IDL.Text,
    'difficulty' : IDL.Nat,
    'size' : IDL.Nat,
    'uncles' : IDL.Vec(IDL.Text),
    'baseFeePerGas' : IDL.Nat,
    'extraData' : IDL.Text,
    'transactionsRoot' : IDL.Opt(IDL.Text),
    'sha3Uncles' : IDL.Text,
    'nonce' : IDL.Nat,
    'number' : IDL.Nat,
    'timestamp' : IDL.Nat,
    'transactions' : IDL.Vec(IDL.Text),
    'gasLimit' : IDL.Nat,
    'logsBloom' : IDL.Text,
    'parentHash' : IDL.Text,
    'gasUsed' : IDL.Nat,
    'mixHash' : IDL.Text,
  });
  return IDL.Service({
    'call_decrease_count' : IDL.Func([], [Result], []),
    'call_increase_count' : IDL.Func([], [Result], []),
    'get_canister_eth_address' : IDL.Func([], [IDL.Text], []),
    'get_count' : IDL.Func([], [Result_1], []),
    'get_latest_ethereum_block' : IDL.Func([], [Block], []),
  });
};
export const init = ({ IDL }) => { return []; };
