use near_sdk::{ext_contract};
use near_sdk::AccountId;
use near_sdk::PromiseOrValue;
use near_sdk::json_types::U128;
use near_contract_standards::non_fungible_token::Token;
use near_contract_standards::non_fungible_token::TokenId;


pub const TGAS: u64 = 1_000_000_000_000;
pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;

// Interface of this contract, for callbacks
#[ext_contract(this_contract)]
trait Callbacks {
  fn nft_supply_for_owner(&self, account_id: AccountId) -> U128;
  fn nft_tokens_for_owner(
    &self,
    account_id: AccountId,
    from_index: Option<U128>,
    limit: Option<u64>
  ) -> Vec<Token>;
  fn nft_transfer_call(
    &mut self, 
    receiver_id: AccountId, 
    token_id: TokenId, 
    approval_id: Option<u64>, 
    memo: Option<String>, 
    msg: String
  ) -> PromiseOrValue<bool>;
}

// Validator interface, for cross-contract calls
#[ext_contract(token_near)]
trait TokenNear {
  fn nft_supply_for_owner(&self, account_id: AccountId) -> U128;
  fn nft_tokens_for_owner(
    &self,
    account_id: AccountId,
    from_index: Option<U128>,
    limit: Option<u64>
  ) -> Vec<Token>;
  fn nft_transfer_call(
    &mut self, 
    receiver_id: AccountId, 
    token_id: TokenId, 
    approval_id: Option<u64>, 
    memo: Option<String>, 
    msg: String
  ) -> PromiseOrValue<bool>;
}