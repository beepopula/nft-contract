use near_contract_standards::non_fungible_token::events::NftTransfer;

use crate::*;



#[near_bindgen]
impl Contract {
    pub fn nft_payout(
        &self, 
        token_id: TokenId,
        balance: U128
    ) -> Payout {
        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
        self.internal_nft_payout(token_series_id, balance.into())
    }
    
    #[payable]
    pub fn nft_transfer_payout(
        &mut self, 
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        balance: Option<U128>,
        max_len_payout: Option<u32>
    ) -> Option<Payout> {
        assert_one_yocto();
    
        let sender_id = env::predecessor_account_id();
        // Transfer
        let previous_token = self.nft_token(token_id.clone()).expect("no token");
        self.tokens.nft_transfer(receiver_id.clone(), token_id.clone(), approval_id, None);
    
        // Payout calculation
        let previous_owner_id = previous_token.owner_id;
        let mut total_perpetual = 0;
        let payout = if let Some(balance) = balance {
            let balance_u128: u128 = u128::from(balance);
            let mut payout: Payout = Payout { payout: HashMap::new() };
    
            let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
            let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
            let royalty = self.token_series_by_id.get(&token_series_id).expect("no type").royalty;
    
            assert!(royalty.len() as u32 <= max_len_payout.unwrap(), "Market cannot payout to that many receivers");
            for (k, v) in royalty.iter() {
                let key = k.clone();
                if key != previous_owner_id {
                    payout.payout.insert(key, royalty_to_payout(*v, balance_u128));
                    total_perpetual += *v;
                }
            }
    
            assert!(
                total_perpetual <= 10000,
                "Total payout overflow"
            );
    
            payout.payout.insert(previous_owner_id.clone(), royalty_to_payout(10000 - total_perpetual, balance_u128));
            Some(payout)
        } else {
            None
        };

        let authorized_id : Option<&AccountId> = if sender_id != previous_owner_id {
            Some(&sender_id)
        } else {
            None
        };

        NftTransfer {
            old_owner_id: &previous_owner_id,
            new_owner_id: &receiver_id,
            token_ids: &[&token_id],
            authorized_id: authorized_id,
            memo: None,
        }
        .emit();
    
        payout
    }
    
    
}

