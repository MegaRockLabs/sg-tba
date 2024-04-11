use cw_utils::PaymentError;
use cosmwasm_std::{ensure, Deps, MessageInfo, Storage, Uint128};
use sg83_base::registry::Cw83TokenRegistryContract;
use sg_std::CosmosMsg;

use crate::{error::ContractError, state::{get_ca_price, get_wl_price, REGISTRY_CONTRACT}};


pub struct WhitelistCollectionAmounts {
    pub to_fair_burn_contract: Uint128,
    pub to_shareholders: Uint128
}

pub struct CreateAccountAmounts {
    pub to_fair_burn_contract: Uint128,
    pub to_collection_owner: Uint128,
    pub to_registry: Uint128,
    pub to_shareholders: Uint128
}



pub fn process_wl_payment(
    deps:   Deps,
    info:   &MessageInfo
) -> Result<WhitelistCollectionAmounts, ContractError> {
    ensure!(info.funds.len() > 0, PaymentError::NoFunds {});
    let fb_fee = Uint128::zero();
    for coin in info.funds.iter() {
        if let Ok(price) = get_wl_price(deps.storage, &coin.denom) {
            let total = fb_fee.checked_add(Uint128::from(price))?;
            if coin.amount >= total {
                return Ok(WhitelistCollectionAmounts {
                    to_fair_burn_contract: fb_fee,
                    to_shareholders: Uint128::from(price)
                });
            } else if info.funds.len() == 1 {
                return Err(ContractError::InsufficientFee(total, coin.amount));
            }
        }
    }
    return Err(ContractError::InsufficienFees{});
}


pub fn process_ca_payment(
    deps:   Deps,
    info:   &MessageInfo,
) -> Result<CreateAccountAmounts, ContractError> {
    ensure!(info.funds.len() > 0, PaymentError::NoFunds {});
    let fb_fee = Uint128::zero();
    let registry = REGISTRY_CONTRACT.load(deps.storage)?;
    let params = Cw83TokenRegistryContract(registry).query_params(deps.querier)?;
    

    for coin in info.funds.iter() {
        if let Ok(price) = get_ca_price(deps.storage, &coin.denom) {
            if let Some(registry_fee) = params
                    .creation_fees.iter()
                    .find(|fee| fee.denom == coin.denom) {
                

                let price = Uint128::from(price);
                let third = price.checked_div(Uint128::from(3u128)).unwrap();

                let total = fb_fee
                        .checked_add(price)?
                        .checked_add(registry_fee.amount)?;
                
                if coin.amount >= total {
        
                    return Ok(CreateAccountAmounts {
                        to_fair_burn_contract: fb_fee,
                        to_collection_owner: third,
                        to_registry: registry_fee.amount,
                        to_shareholders: price - third
                    });

                } else if info.funds.len() == 1 {
                    return Err(ContractError::InsufficientFee(total, coin.amount));
                }
            }
        }
    }
    return Err(ContractError::InsufficienFees{});
}


pub fn shareholder_transfer_msgs(
    _storage: &dyn Storage,
    _amounts: Uint128
) -> Result<Vec<CosmosMsg>, ContractError>  {
    todo!()
}