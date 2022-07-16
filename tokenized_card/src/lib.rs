use near_contract_standards::non_fungible_token::metadata::{
  NFTContractMetadata, NonFungibleTokenMetadataProvider,
};
use near_contract_standards::non_fungible_token::{ TokenId, Token };
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::collections::{LazyOption, UnorderedSet};
use near_sdk::{
  env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, PromiseOrValue, Promise, CryptoHash, Balance,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct TokenizedCard {
  tokens: NonFungibleToken,
  metadata: LazyOption<NFTContractMetadata>,
  total_supply: u128,
  cost_per_token: u128,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
  NonFungibleToken,
  Metadata,
  TokenMetadata,
  Enumeration,
  Approval,
  TokenPerOwnerInner { account_id_hash: CryptoHash },
}

#[near_bindgen]
impl TokenizedCard {
  /// Initializes the contract owned by `owner_id` with metadata, cost_per_token and toal_supply
  #[init]
  pub fn new(
    owner_id: AccountId, 
    metadata: NFTContractMetadata,
    total_supply: u128,
    cost_per_token: u128
  ) -> Self {
    assert!(!env::state_exists(), "Already initialized");
    metadata.assert_valid();
    Self {
      tokens: NonFungibleToken::new(
        StorageKey::NonFungibleToken,
        owner_id,
        Some(StorageKey::TokenMetadata),
        Some(StorageKey::Enumeration),
        Some(StorageKey::Approval),
      ),
      metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
      total_supply,
      cost_per_token,
    }
  }

  #[payable]
  pub fn buy(
    &mut self,
    receiver_id: AccountId,
  ) -> u64 {
    let tokens_minted: u64 = self.tokens.owner_by_id.len();
    let mut attached_deposit = env::attached_deposit();
    let mut count = 1;
    loop {
      if (tokens_minted + count) as u128 > self.total_supply {
        // refund rest amount
        Promise::new(env::predecessor_account_id()).transfer(attached_deposit);
        break;
      }

      let initial_storage_usage = env::storage_usage();
      
      let token_id = format!("TokenizedCard-{}", tokens_minted + count);

      self.internal_add_token_to_owner(&receiver_id, token_id.clone());

      let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;
      let required_cost = env::storage_byte_cost() * Balance::from(required_storage_in_bytes) + self.cost_per_token;

      if required_cost > attached_deposit {
        self.internal_remove_token_from_owner(&receiver_id, token_id.clone());
        if attached_deposit > 1 {
          // refund rest amount
          Promise::new(env::predecessor_account_id()).transfer(attached_deposit);
          break;
        }
      }

      attached_deposit -= required_cost;

      count += 1;
    }

    count - 1
  }

  pub(crate) fn internal_add_token_to_owner(
    &mut self,
    account_id: &AccountId,
    token_id: TokenId,
  ) {
    if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
      let mut tokens_set = tokens_per_owner.get(account_id).unwrap_or_else(|| {
        UnorderedSet::new(
          StorageKey::TokenPerOwnerInner {
              account_id_hash: hash_account_id(&account_id),
          }
          .try_to_vec()
          .unwrap(),
        )
      });
      tokens_set.insert(&token_id);
      tokens_per_owner.insert(account_id, &tokens_set);
    }

    self.tokens.owner_by_id.insert(&token_id, account_id);
  }

  pub(crate) fn internal_remove_token_from_owner(
    &mut self,
    account_id: &AccountId,
    token_id: TokenId,
  ) {
    if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
      let mut tokens_set = tokens_per_owner
        .get(account_id)
        .expect("Token should be owned by the sender");

      tokens_set.remove(&token_id);
      if tokens_set.is_empty() {
          tokens_per_owner.remove(&account_id);
      } else {
          tokens_per_owner.insert(&account_id, &tokens_set);
      }
    }
    
    self.tokens.owner_by_id.remove(&token_id);
  }  
}

near_contract_standards::impl_non_fungible_token_core!(TokenizedCard, tokens);
near_contract_standards::impl_non_fungible_token_approval!(TokenizedCard, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(TokenizedCard, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for TokenizedCard {
  fn nft_metadata(&self) -> NFTContractMetadata {
   self.metadata.get().unwrap()
  }
}


pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
  let mut hash = CryptoHash::default();
  
  hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
  hash
}