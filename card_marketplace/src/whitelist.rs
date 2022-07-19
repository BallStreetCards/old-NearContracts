use crate::*;

impl Contract {
  pub fn allowlist_card(&mut self, nft_contract_id: AccountId, min_price_per_token: U128) {
    //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
    assert_one_yocto();
    self.whitelist.insert(&nft_contract_id, &min_price_per_token);
  }

  pub fn disallow_card(&mut self, nft_contract_id: AccountId) {
    //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
    assert_one_yocto();
    self.whitelist.remove(&nft_contract_id);
  }
}