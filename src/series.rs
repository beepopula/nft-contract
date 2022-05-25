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
    ) -> TokenSeriesId {
        let initial_storage_usage = env::storage_usage();
        let mut caller_id = env::predecessor_account_id();

        if creator_id.is_some() {
            assert_eq!(creator_id.clone().unwrap(), env::signer_account_id(), " signer is not creator_id");
            caller_id = creator_id.unwrap();
        }

        let token_series_id = format!("{}", (self.token_series_by_id.len() + 1));

        assert!(
            self.token_series_by_id.get(&token_series_id).is_none(),
            " duplicate token_series_id"
        );

        let title = token_metadata.title.clone();
        assert!(title.is_some(), " token_metadata.title is required");
        

        let mut total_perpetual = 0;
        let mut total_accounts = 0;
        let royalty_res: HashMap<AccountId, u32> = if let Some(royalty) = royalty {
            for (_ , v) in royalty.iter() {
                total_perpetual += *v;
                total_accounts += 1;
            }
            royalty
        } else {
            HashMap::new()
        };

        assert!(total_accounts <= 10, " royalty exceeds 10 accounts");

        assert!(
            total_perpetual <= 9000,
            "Exceeds maximum royalty -> 9000",
        );

        self.token_series_by_id.insert(&token_series_id, &TokenSeries{
            metadata: token_metadata.clone(),
            creator_id: caller_id.clone(),
            tokens: UnorderedSet::new(
                StorageKey::TokensBySeriesInner {
                    token_series: token_series_id.clone(),
                }
                .try_to_vec()
                .unwrap(),
            ),
            price: None,
            ft_token_id: None,
            is_mintable: true,
            royalty: royalty_res.clone(),
        });

        self.internal_set_price(token_series_id.clone(), mint_price, ft_token_id);

        env::log_str(
            &json!({
                "type": "nft_create_series",
                "params": {
                    "token_series_id": token_series_id,
                    "token_metadata": token_metadata,
                    "creator_id": caller_id,
                    "royalty": royalty_res
                }
            })
            .to_string()
            ,
        );

        refund_extra_storage_deposit(env::storage_usage() - initial_storage_usage, 0);
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
        env::log_str(
            &json!({
                "type": "nft_set_series_non_mintable",
                "params": {
                    "token_series_id": token_series_id,
                }
            })
            .to_string()
            ,
        );
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
        env::log_str(
            &json!({
                "type": "nft_decrease_series_copies",
                "params": {
                    "token_series_id": token_series_id,
                    "copies": U64::from(token_series.metadata.copies.unwrap()),
                    "is_non_mintable": is_non_mintable,
                }
            })
            .to_string()
            ,
        );
        U64::from(token_series.metadata.copies.unwrap())
    }
}


