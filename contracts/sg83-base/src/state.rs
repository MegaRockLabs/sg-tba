use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Map, Item};
use sg_tba::TokenInfo;


#[cw_serde]
pub struct AdminList {
    pub admins: Vec<Addr>,
}

impl AdminList {
    pub fn is_admin(&self, addr: &str) -> bool {
        self.admins.iter().any(|a| a.as_ref() == addr)
    }
}

/// A Mapping of the collections addresses to the number of a tokens with an account
pub static COL_TOKEN_COUNTS  : Map<&str, u32>               = Map::new("c");
/// A Mapping where (collection_address, token_id) => token-bound account address
pub static TOKEN_ADDRESSES   : Map<(&str, &str), String>    = Map::new("t");
/// Cache storage about the token to load from `reply` endpoint when waiting for newly created account address
pub static LAST_ATTEMPTING   : Item<TokenInfo>              = Item::new("l");
/// Code Ids that allowed to be used for token bound accounts
pub static ALLOWED_IDS       : Item<Vec<u64>>               = Item::new("i");
/// A list of the admins addresses
pub static ADMINS            : Item<AdminList>              = Item::new("a");