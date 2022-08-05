use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise, PromiseError, PanicOnDefault};

pub mod external;
pub use crate::external::*;

pub token1: AccountId = "mj-234.tenamint-card.near".parse().unwrap();
pub token2: AccountId = "mj-234.tenamint-card.near".parse().unwrap();

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
  ) -> Promise {
    let mut attached_deposit = env::attached_deposit();
    let mut token1_count = 1;
    let mut token2_count = 1;
    let token1_supply = query_get_supply(token1);
    let token2_supply = query_get_supply(token2);
    let flag = true;

    if flag {
      loop {
        if (token1_count > token1_supply) {
          // refund  rest amount
          Promise::new(env::predecessor_account_id()).transfer(attached_deposit);
          break;
        }

        let initial_storage_usage = env::storage_usage();
        let token_id = query_get_tokenid(token1);
        query_token_transfer(env::predecessor_account_id, token_id);

        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;
        let required_cost = env::storage_byte_cost() * Balance::from(required_storage_in_bytes) + self.cost_per_token;
        
        if required_cost > attached_deposit {
          query_token_transfer(env::current_account_id, token_id);
          if attached_deposit > 1 {
            // refund rest amount
            Promise::new(env::predecessor_account_id()).transfer(attached_deposit);
            break;
          }
        }

        attached_deposit -= required_cost;

        token1_count += 1;
      }

      token1_count -= 1;
    } else {
      loop {
        if ()
      }
    }
  }

  #[private]
  pub fn query_get_supply(token_id: AccountId) -> Promise {
    // Create a promise to call token.nft_supply_for_owner function
    let promise = token_id::ext(token_id.clone())
      .with_static_gas(Gas(5*TGAS))
      .nft_supply_for_owner(env::current_account_id());
    
    return promise.then(
      Self::ext(env::predecessor_account_id())
      .with_static_gas(Gas(5*TGAS))
      .query_get_supply_callback()
    )
  }

  #[private]
  pub fn query_get_supply_callback(#[callback_result] call_result: Result<U128, PromiseError>) -> String {
    // Check if the promise succeeded by calling the method outlined in external.rs
    if call_result.is_err() {
      log!("There was an error contacting {} contract", token_id);
      return "".to_string();
    }

    // return the supply
    let supply: U128 = call_result.unwrap();
    supply
  }

  #[private]
  pub fn query_get_tokenid(token_id: AccountId) -> Promise {
    // Create a promise to call token.nft_tokens_for_owner
    let promise = token_id::ext(token_id.clone())
      .with_static_gas(Gas(5*TGAS))
      .nft_tokens_for_owner(env::current_account_id(), 1, 1)

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
      log!("There was an error contacting {} contract", token_id);
      return "".to_string();
    }

    // return the supply
    let supply: Vec<Token> = call_result.unwrap();
    supply[0].token_id
  }

  #[private]
  pub fn query_token_transfer(receiver_id: AccountId, token_id: TokenId) -> Promise {
    // Create a promise to call token.nft_supply_for_owner function
    let promise = token_id::ext(token_id.clone())
      .with_static_gas(Gas(5*TGAS))
      .nft_transfer_call(receiver_id, token_id, "");
    
    return promise.then(
      Self::ext(env::predecessor_account_id())
      .with_static_gas(Gas(5*TGAS))
      .query_get_supply_callback()
    )
  }

  #[private]
  pub fn query_token_transfer_callback(#[callback_result] call_result: Result<bool, PromiseError>) -> String {
    // Check if the promise succeeded by calling the method outlined in external.rs
    if call_result.is_err() {
      log!("There was an error contacting {} contract", token_id);
      return "".to_string();
    }

    // return the supply
    let result: bool = call_result.unwrap();
    result
  }
}