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
    let mut token1_supply = 
    loop {
      if ()
    }
  }

  #[private]
  pub fn query_get_supply(token_id: AccountId) -> Promise {
    // Create a promise to call token.nft_supply_for_owner function
    let promise = token_id::ext(token_id.clone())
      .with_static_gas(Gas(5*TGAS))
      .nft_supply_for_owner(env::predecessor_account_id());
    
    return promise.then(
      Self::ext(env::current_account_id())
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
}