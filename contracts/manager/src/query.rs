use cosmwasm_std::{StdResult, Deps, Order};
use crate::{msg::CollectionsResponse, state::WL_COLLECTIONS};

const DEFAULT_BATCH_SIZE : u32 = 100;


pub fn collections(
    deps: Deps,
    skip: Option<u32>,
    limit: Option<u32>
) -> StdResult<CollectionsResponse> {
    
    let skip  = skip.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(DEFAULT_BATCH_SIZE) as usize;

    let collections =  WL_COLLECTIONS
        .keys(deps.storage, None, None, Order::Descending)
        .into_iter()
        .enumerate()
        .filter(|(i, _)| *i >= skip)
        .take(limit)
        .map(|(_, c) | c.unwrap())
        .collect::<Vec<String>>();

    Ok(CollectionsResponse { 
        total: collections.len() as u32,
        collections 
    })
}