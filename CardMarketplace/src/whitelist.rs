use crate::*;
use near_sdk::promise_result_as_success;

//struct that holds important information about each sale on the market
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]

impl Contract {
  pub fn allowlistCard(&mut self, nft_contract_id: AccountId, minPricePerToken: U64) {
    //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
    assert_one_yocto();
    self.whitelist.insert(nft_contract_id, minPricePerToken);
  }

  pub fn disallowCard(&mut self, nft_contract_id) {
    //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
    assert_one_yocto();
    self.whitelist.remove(nft_contract_id);
  }
}