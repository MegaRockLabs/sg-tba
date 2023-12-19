use cw_storage_plus::{Map, Item};
use sg_tba::{TokenInfo, RegistryParams};

use crate::msg::FairBurnInfo;

/// A Mapping of the collections addresses to the number of a tokens with an account
pub static COL_TOKEN_COUNTS  : Map<&str, u32>               = Map::new("c");
/// A Mapping where (collection_address, token_id) => token-bound account address
pub static TOKEN_ADDRESSES   : Map<(&str, &str), String>    = Map::new("t");
/// Cache storage about the token to load from `reply` endpoint when waiting for newly created account address
pub static LAST_ATTEMPTING   : Item<TokenInfo>              = Item::new("l");
/// Governance controlled params
pub static SUDO_PARAMS       : Item<RegistryParams>         = Item::new("p");
/// Fee burn developer and burn address info
pub static FAIR_BURN_INFO    : Item<FairBurnInfo>           = Item::new("f");
