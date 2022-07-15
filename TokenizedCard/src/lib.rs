/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use std::collections::HashMap;

    use super::*;

    const MINT_STORAGE_COST: u128 = 5870000000000000000000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn sample_token_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("Olympus Mons".into()),
            description: Some("The tallest mountain in the charted solar system".into()),
            media: None,
            media_hash: None,
            copies: Some(1u64),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.nft_token("1".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "0".to_string();
        let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id.to_string(), accounts(0).to_string());
        assert_eq!(token.metadata.unwrap(), sample_token_metadata());
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), token_id.clone(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(token_id.clone()) {
            assert_eq!(token.token_id, token_id);
            assert_eq!(token.owner_id.to_string(), accounts(1).to_string());
            assert_eq!(token.metadata.unwrap(), sample_token_metadata());
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }

    #[test]
    fn test_revoke() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke(token_id.clone(), accounts(1));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
    }

    #[test]
    fn test_revoke_all() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke_all(token_id.clone());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }
}