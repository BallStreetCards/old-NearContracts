pub trait NonFungibleTokenCore {
  //transfers an NFT to a receiver ID
  fn nft_transfer(
      &mut self,
      receiver_id: AccountId,
      token_id: TokenId,
      memo: Option<String>,
  );

  //transfers an NFT to a receiver and calls a function on the receiver ID's contract
  /// Returns `true` if the token was transferred from the sender's account.
  fn nft_transfer_call(
      &mut self,
      receiver_id: AccountId,
      token_id: TokenId,
      memo: Option<String>,
      msg: String,
  ) -> PromiseOrValue<bool>;

  //get information about the NFT token passed in
  fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;
}

#[ext_contract(ext_non_fungible_token_receiver)]
trait NonFungibleTokenReceiver {
  //Method stored on the receiver contract that is called via cross contract call when nft_transfer_call is called
  /// Returns `true` if the token should be returned back to the sender.
  fn nft_on_transfer(
      &mut self,
      sender_id: AccountId,
      previous_owner_id: AccountId,
      token_id: TokenId,
      msg: String,
  ) -> Promise;
}

#[ext_contract(ext_self)]
trait NonFungibleTokenResolver {
  /*
      resolves the promise of the cross contract call to the receiver contract
      this is stored on THIS contract and is meant to analyze what happened in the cross contract call when nft_on_transfer was called
      as part of the nft_transfer_call method
  */
  fn nft_resolve_transfer(
      &mut self,
      owner_id: AccountId,
      receiver_id: AccountId,
      token_id: TokenId,
  ) -> bool;
}