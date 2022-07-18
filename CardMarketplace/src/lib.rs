use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, Balance, Gas, PanicOnDefault,
    Promise, CryptoHash, BorshStorageKey,
};
use std::collections::HashMap;

pub struct Payout {
  pub payout: HashMap<AccountId, U128>,
}

use crate::external::*;
use crate::internal::*;
use crate::sale::*;
use near_sdk::env::STORAGE_PRICE_PER_BYTE;

mod external;
mod internal;
mod nft_callbacks;
mod sale;
mod sale_views;

//GAS constants to attach to calls
const GAS_FOR_RESOLVE_PURCHASE: Gas = Gas(115_000_000_000_000);
const GAS_FOR_NFT_TRANSFER: Gas = Gas(15_000_000_000_000);

//the minimum storage to have a sale on the contract.
const STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;

//every sale will have a unique ID which is `CONTRACT + DELIMITER + TOKEN_ID`
static DELIMETER: &str = ".";

//Creating custom types to use within the contract. This makes things more readable. 
pub type SalePriceInYoctoNear = U128;
pub type TokenId = String;
pub type FungibleTokenId = AccountId;
pub type ContractAndTokenId = String;
//defines the payout type we'll be parsing from the NFT contract as a part of the royalty standard.

//main contract struct to store all the information
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]



pub struct Contract{
  //keep track of the owner of the contract
  pub owner_id: AccountId,
    
  /*
      to keep track of the sales, we map the ContractAndTokenId to a Sale. 
      the ContractAndTokenId is the unique identifier for every sale. It is made
      up of the `contract ID + DELIMITER + token ID`
  */
  pub sales: UnorderedMap<ContractAndTokenId, Sale>,
  
  //keep track of all the Sale IDs for every account ID
  pub by_owner_id: LookupMap<AccountId, UnorderedSet<ContractAndTokenId>>,

  //keep track of all the token IDs for sale for a given contract
  pub by_nft_contract_id: LookupMap<AccountId, UnorderedSet<TokenId>>,

  //keep track of the storage that accounts have payed
  pub storage_deposits: LookupMap<AccountId, Balance>,
}

