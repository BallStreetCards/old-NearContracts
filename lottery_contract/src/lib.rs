use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise, PromiseError, PanicOnDefault, Balance};
use near_sdk::json_types::U128;
use near_contract_standards::non_fungible_token::{Token, TokenId};

pub mod external;
pub use crate::external::*;

pub const token1: AccountId = "mj-234.tenamint-card.near".parse().unwrap();
pub const token2: AccountId = "mj-234.tenamint-card.near".parse().unwrap();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
  pub cost_per_token: u128,
  pub accepted_token: AccountId,
}

#[near_bindgen]
impl Contract {
  #[init]
  #[private] // Public - but only callable by env::current_account_id()
  pub fn new(cost_per_token: u128, accepted_token: AccountId) -> Self {
    assert!(!env::state_exists(), "Already initialized");
    Self {
      cost_per_token,
      accepted_token,
    }
  }

  #[payable]
  pub fn buy(
    &mut self,
    receiver_id: AccountId,
  ) -> (u128, u128) {
    
    let mut attached_deposit = env::attached_deposit();
    let mut token1_count = 1;
    let mut token2_count = 1;
    let token1_supply = self.query_get_supply(token1);
    let token2_supply = self.query_get_supply(token2);
    let flag = true;

    let random_seed: Vec<u8> = env::random_seed();
    if random_seed[0] % 2 == 1 {flag = true}
    if random_seed[0] % 2 == 0 {flag = false}

    loop {
      if ((token1_count + token2_count)> (token1_supply + token2_supply)) {
        // refund  rest amount
        Promise::new(env::predecessor_account_id()).transfer(attached_deposit);
        break;
      }
      if (token1_count > token1_supply) {flag = false;}
      if (token2_count > token2_supply) {flag = true;}

      let initial_storage_usage = env::storage_usage();
      let mut token_id;
      if flag {
        token_id = self.query_get_tokenid(token1);
        self.query_token_transfer(token1, env::predecessor_account_id(), token_id);
      } else {
        token_id = self.query_get_tokenid(token2);
        self.query_token_transfer(token2, env::predecessor_account_id(), token_id);
      }

      let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;
      let required_cost = env::storage_byte_cost() * Balance::from(required_storage_in_bytes) + self.cost_per_token;
      
      if required_cost > attached_deposit {
        if flag {
          self.query_token_transfer(token1, env::current_account_id(), token_id);
        } else {
          self.query_token_transfer(token2, env::current_account_id(), token_id);
        }
        if attached_deposit > 1 {
          // refund rest amount
          Promise::new(env::predecessor_account_id()).transfer(attached_deposit);
          break;
        }
      }

      attached_deposit -= required_cost;

      if flag {
        token1_count += 1;
      } else {
        token2_count += 1;
      }
      flag = !flag;
    }

    token1_count -= 1;
    token2_count -= 1;
    (token1_count, token2_count)
  }

  #[private]
  pub(crate) fn query_get_supply(&mut self, token: AccountId) -> Promise {
    // Create a promise to call token.nft_supply_for_owner function
    let promise = token_near::ext(token.clone())
      .with_static_gas(Gas(5*TGAS))
      .nft_supply_for_owner(env::current_account_id());
    
    return promise.then(
      Self::ext(env::predecessor_account_id())
      .with_static_gas(Gas(5*TGAS))
      .query_get_supply_callback()
    )
  }

  #[private]
  pub fn query_get_supply_callback(#[callback_result] call_result: Result<U128, PromiseError>) -> u128 {
    // Check if the promise succeeded by calling the method outlined in external.rs
    if call_result.is_err() {
      log!("There was an error contacting get_supply contract");
      return 0;
    }

    // return the supply
    let supply: U128 = call_result.unwrap();
    supply.0
  }

  #[private]
  pub(crate) fn query_get_tokenid(&mut self, token: AccountId) -> Promise {
    // Create a promise to call token.nft_tokens_for_owner
    let promise = token_near::ext(token.clone())
      .with_static_gas(Gas(5*TGAS))
      .nft_tokens_for_owner(env::current_account_id(), Some(1), Some(1));

    return promise.then(
      Self::ext(env::predecessor_account_id())
      .with_static_gas(Gas(5*TGAS))
      .query_get_tokenid_callback()
    )
  }

  #[private]
  pub fn query_get_tokenid_callback(#[callback_result] call_result: Result<Vec<Token>, PromiseError>) -> String {
    // Check if the promise succeeded by calling the method outlined in external.rs
    if call_result.is_err() {
      log!("There was an error contacting get_tokenid function in contract");
      return "".to_string();
    }

    // return the supply
    let supply: Vec<Token> = call_result.unwrap();
    supply[0].token_id
  }

  #[private]
  pub(crate) fn query_token_transfer(&mut self, token: AccountId, receiver_id: AccountId, token_id: TokenId) -> Promise {
    // Create a promise to call token.nft_supply_for_owner function
    let promise = token_near::ext(token.clone())
      .with_static_gas(Gas(5*TGAS))
      .nft_transfer_call(receiver_id, token_id, None, None, "".to_string());
    
    return promise.then(
      Self::ext(env::predecessor_account_id())
      .with_static_gas(Gas(5*TGAS))
      .query_get_supply_callback()
    )
  }

  #[private]
  pub fn query_token_transfer_callback(#[callback_result] call_result: Result<bool, PromiseError>) -> bool {
    // Check if the promise succeeded by calling the method outlined in external.rs
    if call_result.is_err() {
      log!("There was an error contacting token_transfer function in contacting");
      return false;
    }

    // return the supply
    let result: bool = call_result.unwrap();
    result
  }
}