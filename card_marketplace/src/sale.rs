use crate::*;
use near_sdk::promise_result_as_success;

//struct that holds important information about each sale on the market
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
  //owner of the sale
  pub owner_id: AccountId,
  //market contract's approval ID to transfer the token on behalf of the owner
  pub approval_id: u64,
  //nft contract where the token was minted
  pub nft_contract_id: String,
  //actual token ID for sale
  pub token_id: String,
  //sale price in yoctoNEAR that the token is listed for
  pub sale_conditions: SalePriceInYoctoNear,
}

#[near_bindgen]
impl Contract {

  //removes a sale from the market. 
  #[payable]
  pub fn unlist(&mut self, nft_contract_id: AccountId, token_id: String) {
    //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
    assert_one_yocto();
    //get the sale object as the return value from removing the sale internally
    let sale = self.internal_remove_sale(nft_contract_id.into(), token_id);
    //get the predecessor of the call and make sure they're the owner of the sale
    let owner_id = env::predecessor_account_id();
    //if this fails, the remove sale will revert
    assert_eq!(owner_id, sale.owner_id, "Must be sale owner");
  }

  //updates the price for a sale on the market
  #[payable]
  pub fn list(&mut self, nft_contract_id: AccountId, token_id: String, price: U128) {
    //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
    assert_one_yocto();

    //get the min price from the whitelist. If there is no result, panic. 
    let min_price = self.whitelist.get(&nft_contract_id).expect("No Whitelist");

    //make sure the setting price is greater than the min price
    assert!(price >= min_price, "Setted price must be greater than or equal to the min price: {:?}", min_price);

    //get the ID from the nft contract and token
    let contract_and_token_id = format!("{}{}{}", nft_contract_id, DELIMETER, token_id);
    //create new Sate item
    let sale = Sale {
      owner_id: env::predecessor_account_id(),
      approval_id: self.sales.len() + 1,
      nft_contract_id: nft_contract_id.into(),
      token_id: token_id,
      sale_conditions: price,
    };
    //inseret new item to sales list
    self.sales.insert(&contract_and_token_id, &sale);
  }
  
  //updates the price for a sale on the market
  #[payable]
  pub fn update_price(&mut self, nft_contract_id: AccountId, token_id: String, price: U128) {
    assert_one_yocto();

    //create the unique sale ID from the nft contract and token
    let contract_id: AccountId = nft_contract_id.into();
    let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);

    //get the sale object from the unique sale ID. If there is no token, panic. 
    let mut sale = self.sales.get(&contract_and_token_id).expect("No Sale");

    //assert that the caller of the function is the sale owner
    assert_eq!(env::predecessor_account_id(), sale.owner_id, "Must be sale owner");
    //set the sale conditions equal to the passed in price
    sale.sale_conditions = price;
    //insert the sale back into the map for the unique sale ID
    self.sales.insert(&contract_and_token_id, &sale);
  }

  //place an offer on a specific sale. The sale will go through as long as your deposit is greater than or equal to the list price
  #[payable]
  pub fn buy(&mut self, nft_contract_id: AccountId, token_id: String) {
     //get the attached deposit and make sure it's greater than 0
    let deposit = env::attached_deposit();
    assert!(deposit > 0, "Attached deposit must be greater than 0");

     //convert the nft_contract_id from a AccountId to an AccountId
    let contract_id: AccountId = nft_contract_id.into();
    //get the unique sale ID (contract + DELIMITER + token ID)
    let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);

    //get the sale object from the unique sale ID. If the sale doesn't exist, panic.
    let sale = self.sales.get(&contract_and_token_id).expect("No sale");

    //get the buyer ID which is the person who called the function and make sure they're not the owner of the sale
    let buyer_id = env::predecessor_account_id();
    assert_ne!(sale.owner_id, buyer_id, "Cannot bid on your own sale."); 

    //(dot 0 converts from U128 to u128)
    //get the u128 price of the token (dot 0 converts from U128 to u128)
    let price = sale.sale_conditions.0;

    //make sure the deposit is greater than the price
    assert!(deposit >= price, "Attached deposit must be greater than or equal to the current price: {:?}", price);
    
    //process the purchase (which will remove the sale, transfer and get the payout from the nft contract, and then distribute royalties) 
    self.process_purchase(
      contract_id,
      token_id,
      U128(deposit),
      buyer_id,
    );
  }

  //private function used when a sale is purchased. 
    //this will remove the sale, transfer and get the payout from the nft contract, and then distribute royalties
  #[private]
  pub fn process_purchase(&mut self, nft_contract_id: AccountId, token_id: String, price: U128, buyer_id: AccountId) {
     //get the sale object by removing the sale
    let sale = self.internal_remove_sale(nft_contract_id.clone(), token_id.clone());

    //initiate a cross contract call to the nft contract. This will transfer the token to the buyer and return
    //a payout object used for the market to distribute funds to the appropriate accounts.
    ext_contract::ext(nft_contract_id)
      .with_attached_deposit(1)
      .with_static_gas(GAS_FOR_NFT_TRANSFER)
      .nft_transfer_payout(
        buyer_id.clone(), 
        token_id, 
        sale.approval_id, 
        "payout from market".to_string(),
        price,
        10
      ).then(
        Self::ext(env::current_account_id())
          .with_static_gas(GAS_FOR_RESOLVE_PURCHASE)
          .resolve_purchase(
            buyer_id,
            price
          )
      );
  }

  /*
  private method used to resolve the promise when calling nft_transfer_payout. This will take the payout object and 
  check to see if it's authentic and there's no problems. If everything is fine, it will pay the accounts. If there's a problem,
  it will refund the buyer for the price. 
  */
  #[private]
  pub fn resolve_purchase(&mut self, buyer_id: AccountId, price: U128) -> U128 {
    // checking for payout information returned from the nft_transfer_payout method
    let payout_option = promise_result_as_success().and_then(|value| {
      near_sdk::serde_json::from_slice::<Payout>(&value)
        .ok()
        .and_then(|payout_object| {
          //we'll check if length of the payout object is > 10 or it's empty. In either case, we return None
          if payout_object.payout.len() > 10 || payout_object.payout.is_empty() {
            env::log_str("Cannot have more than 10 royalties");
            None
          } else {
            let mut remainder = price.0;
            for &value in payout_object.payout.values() {
              remainder = remainder.checked_sub(value.0)?;
            }
            if remainder == 0 || remainder == 1 {
              Some(payout_object.payout)
            } else {
              None
            }
          }
        })
    });

    // if the payout option was some payout, we set this payout variable equal to that some payout
    let payout = if let Some(payout_option) = payout_option {
      payout_option
    } else {
      Promise::new(buyer_id).transfer(u128::from(price));
      return price;
    };

    // NEAR payouts
    for (receiver_id, amount) in payout {
      let fee_amount = amount.0 * self.fee.0 / 100;
      Promise::new(self.fee_recipient.clone()).transfer(fee_amount);
      Promise::new(receiver_id).transfer(amount.0 - fee_amount);
    }

    //return the price payout out
    price
  }
}

//this is the cross contract call that we call on our own contract. 
/*
  private method used to resolve the promise when calling nft_transfer_payout. This will take the payout object and 
  check to see if it's authentic and there's no problems. If everything is fine, it will pay the accounts. If there's a problem,
  it will refund the buyer for the price. 
*/
#[ext_contract(ext_self)]
trait ExtSelf {
    fn resolve_purchase(
        &mut self,
        buyer_id: AccountId,
        price: U128,
    ) -> Promise;
}