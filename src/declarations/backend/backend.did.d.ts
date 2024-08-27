import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Block {
  'miner' : string,
  'totalDifficulty' : bigint,
  'receiptsRoot' : string,
  'stateRoot' : string,
  'hash' : string,
  'difficulty' : bigint,
  'size' : bigint,
  'uncles' : Array<string>,
  'baseFeePerGas' : bigint,
  'extraData' : string,
  'transactionsRoot' : [] | [string],
  'sha3Uncles' : string,
  'nonce' : bigint,
  'number' : bigint,
  'timestamp' : bigint,
  'transactions' : Array<string>,
  'gasLimit' : bigint,
  'logsBloom' : string,
  'parentHash' : string,
  'gasUsed' : bigint,
  'mixHash' : string,
}
export type Result = { 'Ok' : string } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : bigint } |
  { 'Err' : string };
export interface _SERVICE {
  'call_decrease_count' : ActorMethod<[], Result>,
  'call_increase_count' : ActorMethod<[], Result>,
  'get_canister_eth_address' : ActorMethod<[], string>,
  'get_count' : ActorMethod<[], Result_1>,
  'get_latest_ethereum_block' : ActorMethod<[], Block>,
  'get_stored_transaction_hashes' : ActorMethod<[], Array<string>>,
  'store_transaction_hash' : ActorMethod<[string], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
