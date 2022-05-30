use near_sdk::log;

use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_create_series(
        &mut self,
        creator_id: Option<AccountId>,
        token_metadata: TokenMetadata,
        mint_price: Option<U128>, 
        ft_token_id: Option<AccountId>,
        royalty: Option<HashMap<AccountId, u32>>,
        notify_contract_id: Option<AccountId>
    ) -> TokenSeriesId {
        let initial_storage_usage = env::storage_usage();
        let mut caller_id = env::predecessor_account_id();

        if creator_id.is_some() {
            assert_eq!(creator_id.clone().unwrap(), env::signer_account_id(), " signer is not creator_id");
            caller_id = creator_id.unwrap();
        }

        let token_series_id = self.internal_create_series(caller_id.clone(), token_metadata.clone(), mint_price, ft_token_id.clone(), royalty);
        self.internal_set_price(token_series_id.clone(), mint_price, ft_token_id);

        if mint_price.is_none() && token_metadata.copies == Some(1)  {
            self.accounts.insert(&caller_id, &(env::attached_deposit() - (env::storage_usage() - initial_storage_usage) as u128 * env::storage_byte_cost() as u128));
            let token_id = self.internal_nft_mint_series(caller_id.clone(), token_series_id.clone(), caller_id.clone());
            NftMint { owner_id: &caller_id, token_ids: &[&token_id], memo: None }.emit();
        } else {
            refund_extra_storage_deposit(env::storage_usage() - initial_storage_usage, 0);
        }
        if notify_contract_id.is_some() {
            let args_json = json!({
                "token_series_id": token_series_id
            }).to_string();
            Promise::new(notify_contract_id.unwrap()).function_call("add_item".to_string(), json!({
                "args": args_json
            }).to_string().as_bytes().to_vec(), 0, (env::prepaid_gas() - env::used_gas()) / 2);
        }
        
        token_series_id
    }

    #[payable]
    pub fn nft_set_series_price(&mut self, token_series_id: TokenSeriesId, mint_price: Option<U128>, ft_token_id: Option<AccountId>) -> Option<U128> {
        assert_one_yocto();
        let token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
        assert_eq!(
            env::predecessor_account_id(),
            token_series.creator_id,
            " Creator only"
        );

        assert_eq!(
            token_series.is_mintable,
            true,
            " token series is not mintable"
        );
        self.internal_set_price(token_series_id, mint_price, ft_token_id)
    }

    #[payable]
    pub fn nft_set_series_non_mintable(&mut self, token_series_id: TokenSeriesId) {
        assert_one_yocto();

        let mut token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
        assert_eq!(
            env::predecessor_account_id(),
            token_series.creator_id,
            " Creator only"
        );

        assert_eq!(
            token_series.is_mintable,
            true,
            " already non-mintable"
        );

        assert_eq!(
            token_series.metadata.copies,
            None,
            " decrease supply if copies not null"
        );

        token_series.is_mintable = false;
        self.token_series_by_id.insert(&token_series_id, &token_series);
    }

    #[payable]
    pub fn nft_decrease_series_copies(
        &mut self, 
        token_series_id: TokenSeriesId, 
        decrease_copies: U64
    ) -> U64 {
        assert_one_yocto();

        let mut token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
        assert_eq!(
            env::predecessor_account_id(),
            token_series.creator_id,
            " Creator only"
        );

        let minted_copies = token_series.tokens.len();
        let copies = token_series.metadata.copies.unwrap();

        assert!(
            (copies - decrease_copies.0) >= minted_copies,
            " cannot decrease supply, already minted : {}", minted_copies
        );

        let is_non_mintable = if (copies - decrease_copies.0) == minted_copies {
            token_series.is_mintable = false;
            true
        } else {
            false
        };

        token_series.metadata.copies = Some(copies - decrease_copies.0);

        self.token_series_by_id.insert(&token_series_id, &token_series);
        U64::from(token_series.metadata.copies.unwrap())
    }
}


