import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Account {
  'id' : bigint,
  'holder_name' : string,
  'balance' : number,
  'created_at' : bigint,
}
export type Error = { 'NotFound' : { 'msg' : string } } |
  { 'InsufficientFunds' : { 'msg' : string } };
export type Result = { 'Ok' : null } |
  { 'Err' : Error };
export type Result_1 = { 'Ok' : Account } |
  { 'Err' : Error };
export type Result_2 = { 'Ok' : number } |
  { 'Err' : Error };
export type Result_3 = { 'Ok' : bigint } |
  { 'Err' : Error };
export type Result_4 = { 'Ok' : Array<Transaction> } |
  { 'Err' : Error };
export type Result_5 = { 'Ok' : Transaction } |
  { 'Err' : Error };
export interface Transaction {
  'receiver_id' : bigint,
  'sender_id' : bigint,
  'timestamp' : bigint,
  'amount' : number,
}
export interface TransferPayload {
  'receiver_id' : bigint,
  'sender_id' : bigint,
  'amount' : number,
}
export interface _SERVICE {
  'create_account' : ActorMethod<[string, number], [] | [Account]>,
  'delete_account' : ActorMethod<[bigint], Result>,
  'get_account' : ActorMethod<[bigint], Result_1>,
  'get_account_balance' : ActorMethod<[bigint], Result_2>,
  'get_account_created_at' : ActorMethod<[bigint], Result_3>,
  'get_all_accounts' : ActorMethod<[], Array<Account>>,
  'get_all_transactions' : ActorMethod<[], Result_4>,
  'get_receiver_account' : ActorMethod<[bigint], Result_1>,
  'get_sender_account' : ActorMethod<[bigint], Result_1>,
  'transfer_funds' : ActorMethod<[TransferPayload], Result_5>,
  'update_account_holder_name' : ActorMethod<[bigint, string], Result>,
}
