
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

use crate::*;

impl Contract {
    pub(crate) fn internal_nft_mint_series(
        &mut self, 
        sender_id: AccountId,
        token_series_id: TokenSeriesId, 
        receiver_id: AccountId
    ) -> TokenId {
        let account = self.accounts.get(&sender_id);
        assert!(account.is_some(), "not registered");
        let refund = account.unwrap() - self.tokens.extra_storage_in_bytes_per_token as u128 * env::storage_byte_cost();
        assert!(refund > 0, "not enough deposit");
        let mut token_series = self.token_series_by_id.get(&token_series_id).expect(" Token series not exist");
        assert!(
            token_series.is_mintable,
            " Token series is not mintable"
        );
    
        let num_tokens = token_series.tokens.len();
        let max_copies = token_series.metadata.copies.unwrap_or(u64::MAX);
        assert!(num_tokens < max_copies, "Series supply maxed");
    
        if (num_tokens + 1) >= max_copies {
            token_series.is_mintable = false;
        }
    
        let token_id = format!("{}{}{}", &token_series_id, TOKEN_DELIMETER, num_tokens + 1);
        token_series.tokens.insert(&token_id);
        self.token_series_by_id.insert(&token_series_id, &token_series);
    
        // you can add custom metadata to each token here
        let metadata = Some(token_series.metadata);
    
        //let token = self.tokens.mint(token_id, receiver_id, metadata);
        // From : https://github.com/near/near-sdk-rs/blob/master/near-contract-standards/src/non_fungible_token/core/core_impl.rs#L359
        // This allows lazy minting
    
        let owner_id: AccountId = receiver_id.clone();
        self.tokens.owner_by_id.insert(&token_id, &owner_id);
    
        self.tokens
            .token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.insert(&token_id, &metadata.as_ref().unwrap()));
    
         if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
             let mut token_ids = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                 UnorderedSet::new(StorageKey::TokensPerOwner {
                     account_hash: env::sha256(&owner_id.as_bytes()),
                 })
             });
             token_ids.insert(&token_id);
             tokens_per_owner.insert(&owner_id, &token_ids);
        }
        self.accounts.insert(&sender_id, &refund);
        token_id
    }

    pub(crate) fn internal_mint_with_token(&mut self, sender_id: AccountId, ft_token_id: AccountId, amount: u128, token_series_id: TokenSeriesId, receiver_id: AccountId) -> (TokenId, Balance) {
        let account = self.accounts.get(&sender_id);
        assert!(account.is_some(), "not registered");
        
        let token_series = self.token_series_by_id.get(&token_series_id).expect(" Token series not exist");
        let price: u128 = token_series.price.expect(" not for sale");
        assert!(token_series.ft_token_id.clone().unwrap() == ft_token_id, "uncorrect token");
        assert!(
            amount >= price,
            " amount is less than price : {}",
            price
        );
        
        let token_id = self.internal_nft_mint_series(sender_id, token_series_id, receiver_id);
        (token_id, amount - price)
    }

    pub(crate) fn internal_mint_with_near(&mut self, token_series_id: TokenSeriesId, receiver_id: AccountId) -> TokenId {
        let sender_id = env::predecessor_account_id();
        let amount = self.accounts.get(&sender_id).unwrap();
        let token_series = self.token_series_by_id.get(&token_series_id).expect(" Token series not exist");
        let price: u128 = token_series.price.expect(" not for sale");
        self.accounts.insert(&sender_id, &(amount - price));
        let token_id = self.internal_nft_mint_series(sender_id, token_series_id, receiver_id);
        token_id
    }

    pub(crate) fn internal_set_price(&mut self, token_series_id: TokenSeriesId, mint_price: Option<U128>, ft_token_id: Option<AccountId>) -> Option<U128> {
        let mut token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");

        if mint_price.is_none() || ft_token_id.is_none() {
            token_series.price = None;
            token_series.ft_token_id = None;
        } else {
            token_series.price = Some(mint_price.unwrap().into());
            token_series.ft_token_id = Some(ft_token_id.clone().unwrap());
        }

        self.token_series_by_id.insert(&token_series_id, &token_series);
        env::log_str(
            &json!({
                "type": "nft_set_series_price",
                "params": {
                    "token_series_id": token_series_id,
                    "mint price": mint_price,
                    "ft_token_id": ft_token_id
                }
            })
            .to_string()
            ,
        );
        if let Some(price) = mint_price { Some(price) } else { None }
    }

    pub(crate) fn internal_nft_payout(&self, token_series_id: TokenId, balance: u128) -> Payout {
        let token_series = self.token_series_by_id.get(&token_series_id).expect("no type");
        let royalty = token_series.royalty;
        let balance_u128: u128 = balance.into();
    
        let mut payout: Payout = Payout { payout: HashMap::new() };
        let mut total_perpetual = 0;
    
        for (k, v) in royalty.iter() {
            let key = k.clone();
            payout.payout.insert(key, royalty_to_payout(*v, balance_u128));
            total_perpetual += *v;
            
        }
        payout.payout.insert(token_series.creator_id, royalty_to_payout(10000 - total_perpetual, balance_u128));
        payout
    }

    

}

