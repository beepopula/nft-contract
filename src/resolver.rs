
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::serde_json;

use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenReceiverMessage {
    token_series_id: TokenSeriesId,
    receiver_id: AccountId
}

#[near_bindgen]
#[allow(unreachable_code)]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    /// `msg` format is either "" for deposit or `TokenReceiverMessage`.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_in = env::predecessor_account_id();
        if msg.is_empty() {
            panic!("no msg found")
        } else {
            let message = serde_json::from_str::<TokenReceiverMessage>(&msg).expect("ERR_MSG_WRONG_FORMAT");
            let info = self.internal_mint_with_token(sender_id, token_in, amount.into(), message.token_series_id, message.receiver_id.clone());
            NftMint { owner_id: &message.receiver_id, token_ids: &[&info.0], memo: None }.emit();
            PromiseOrValue::Value(U128(info.1))
        }
    }
}