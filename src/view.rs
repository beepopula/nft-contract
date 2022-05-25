use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenSeriesInfo {
    token_series_id: TokenSeriesId,
	metadata: TokenMetadata,
	creator_id: AccountId,
    royalty: HashMap<AccountId, u32>,
    ft_token_id: Option<AccountId>,
    mint_price: Option<U128>
}

#[near_bindgen]
impl Contract {
    pub fn get_owner(&self) -> AccountId {
        self.tokens.owner_id.clone()
    }

    pub fn nft_get_series_single(&self, token_series_id: TokenSeriesId) -> TokenSeriesInfo {
        let token_series = self.token_series_by_id.get(&token_series_id).expect("Series does not exist");
        let price = match token_series.price {
            Some(p) => Some(U128::from(p)),
            None =>  None
        };
        TokenSeriesInfo{
            token_series_id,
            metadata: token_series.metadata,
            creator_id: token_series.creator_id,
            royalty: token_series.royalty,
            mint_price: price,
            ft_token_id: token_series.ft_token_id
        }
    }
    
    pub fn nft_get_series_format(self) -> (char, &'static str, &'static str) {
        (TOKEN_DELIMETER, TITLE_DELIMETER, EDITION_DELIMETER)
    }
    
    pub fn nft_get_series_price(self, token_series_id: TokenSeriesId) -> Option<U128> {
        let price = self.token_series_by_id.get(&token_series_id).unwrap().price;
        match price {
            Some(p) => return Some(U128::from(p)),
            None => return None
        };
    }
    
    pub fn nft_get_series(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<TokenSeriesInfo> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.token_series_by_id.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
    
        self.token_series_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(token_series_id, token_series)| {
                let price = match token_series.price {
                    Some(p) => Some(U128::from(p)),
                    None =>  None
                };
                TokenSeriesInfo{
                    token_series_id,
                    metadata: token_series.metadata,
                    creator_id: token_series.creator_id,
                    royalty: token_series.royalty,
                    mint_price: price,
                    ft_token_id: token_series.ft_token_id
                }
            }
            )
            .collect()
    }
    
    pub fn nft_supply_for_series(&self, token_series_id: TokenSeriesId) -> U64 {
        self.token_series_by_id.get(&token_series_id).expect("Token series not exist").tokens.len().into()
    }
    
    pub fn nft_tokens_by_series(
        &self,
        token_series_id: TokenSeriesId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        let tokens = self.token_series_by_id.get(&token_series_id).unwrap().tokens;
        assert!(
            (tokens.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
    
        tokens
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_id| self.nft_token(token_id).unwrap())
            .collect()
    }

    pub fn get_storage_fee(&self) -> U128 {
        (self.tokens.extra_storage_in_bytes_per_token as u128 * env::storage_byte_cost()).into()
    }
}
