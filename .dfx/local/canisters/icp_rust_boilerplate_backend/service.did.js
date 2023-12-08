export const idlFactory = ({ IDL }) => {
  const Account = IDL.Record({
    'id' : IDL.Nat64,
    'holder_name' : IDL.Text,
    'balance' : IDL.Float64,
    'created_at' : IDL.Nat64,
  });
  const Error = IDL.Variant({
    'NotFound' : IDL.Record({ 'msg' : IDL.Text }),
    'InsufficientFunds' : IDL.Record({ 'msg' : IDL.Text }),
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : Error });
  const Result_1 = IDL.Variant({ 'Ok' : Account, 'Err' : Error });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Float64, 'Err' : Error });
  const Result_3 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : Error });
  const Transaction = IDL.Record({
    'receiver_id' : IDL.Nat64,
    'sender_id' : IDL.Nat64,
    'timestamp' : IDL.Nat64,
    'amount' : IDL.Float64,
  });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Vec(Transaction), 'Err' : Error });
  const TransferPayload = IDL.Record({
    'receiver_id' : IDL.Nat64,
    'sender_id' : IDL.Nat64,
    'amount' : IDL.Float64,
  });
  const Result_5 = IDL.Variant({ 'Ok' : Transaction, 'Err' : Error });
  return IDL.Service({
    'create_account' : IDL.Func(
        [IDL.Text, IDL.Float64],
        [IDL.Opt(Account)],
        [],
      ),
    'delete_account' : IDL.Func([IDL.Nat64], [Result], []),
    'get_account' : IDL.Func([IDL.Nat64], [Result_1], ['query']),
    'get_account_balance' : IDL.Func([IDL.Nat64], [Result_2], ['query']),
    'get_account_created_at' : IDL.Func([IDL.Nat64], [Result_3], ['query']),
    'get_all_accounts' : IDL.Func([], [IDL.Vec(Account)], ['query']),
    'get_all_transactions' : IDL.Func([], [Result_4], ['query']),
    'get_receiver_account' : IDL.Func([IDL.Nat64], [Result_1], ['query']),
    'get_sender_account' : IDL.Func([IDL.Nat64], [Result_1], ['query']),
    'transfer_funds' : IDL.Func([TransferPayload], [Result_5], []),
    'update_account_holder_name' : IDL.Func(
        [IDL.Nat64, IDL.Text],
        [Result],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
