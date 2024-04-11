use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

/// Registry Contract Address
pub static  REGISTRY_CONTRACT   : Item<Addr>             = Item::new("r");
/// Mapping of the whitelisted collections addresses to the number of a tokens with an account
pub static  WL_COLLECTIONS      : Map<&str, String>        = Map::new("c");


/// Prices for whitelistig a collection
const WL_PRICES           : Map<&str, u128>          = Map::new("p");
/// Prices of creating an account
const CA_PRICES           : Map<&str, u128>          = Map::new("a");


pub fn get_wl_prices(
    storage: &dyn Storage
) -> StdResult<Vec<(String, u128)>> {
    WL_PRICES
    .range(storage, None, None, Order::Ascending)
    .collect()
}

pub fn get_wl_price(
    storage: &dyn Storage,
    denom: &str
) -> StdResult<u128> {
    WL_PRICES.load(storage, denom)
}


pub fn save_wl_prices(
    storage: &mut dyn Storage,
    prices: Vec<(String, Uint128)>
) -> StdResult<()> {
    for (denom, price) in prices {
        WL_PRICES.save(storage, denom.as_ref(), &price.u128())?;
    }
    Ok(())
}

pub fn save_wl_price(
    storage: &mut dyn Storage,
    denom: &str,
    price: u128
) -> StdResult<()> {
    WL_PRICES.save(storage, denom, &price)
}


pub fn get_ca_prices(
    storage: &dyn Storage
) -> StdResult<Vec<(String, u128)>> {
    CA_PRICES
    .range(storage, None, None, Order::Ascending)
    .collect()
}

pub fn get_ca_price(
    storage: &dyn Storage,
    denom: &str
) -> StdResult<u128> {
    CA_PRICES.load(storage, denom)
}


pub fn save_ca_prices(
    storage: &mut dyn Storage,
    prices: Vec<(String, Uint128)>
) -> StdResult<()> {
    for (denom, price) in prices {
        CA_PRICES.save(storage, denom.as_ref(), &price.u128())?;
    }
    Ok(())
}

pub fn save_ca_price(
    storage: &mut dyn Storage,
    denom: &str,
    price: u128
) -> StdResult<()> {
    CA_PRICES.save(storage, denom, &price)
}
