use cosmwasm_std::{ensure, Coin, Deps, MessageInfo, Uint128};
use cw_utils::PaymentError;
use sg83_base::registry::Cw83TokenRegistryContract;

use crate::{error::ContractError, state::{get_ca_price, REGISTRY_CONTRACT}};

pub fn process_ca_payments(
    deps:   Deps,
    info:   &MessageInfo,
) -> Result<(), ContractError> {
    ensure!(info.funds.len() > 0, PaymentError::NoFunds {});
    let fb_fee = Uint128::zero();
    let registry = REGISTRY_CONTRACT.load(deps.storage)?;
    let params = Cw83TokenRegistryContract(registry).query_params(deps.querier)?;
    
    let mut coin_to_pay : Coin;

    for coin in info.funds.iter() {
        if let Ok(price) = get_ca_price(deps.storage, &coin.denom) {
            if let Some(registry_fee) = params
                    .creation_fees.iter()
                    .find(|fee| fee.denom == coin.denom) {
                
                let total = fb_fee
                        .checked_add(Uint128::from(price))?
                        .checked_add(registry_fee.amount)?;
                
                if coin.amount >= total {
                    coin_to_pay = Coin {
                        denom: coin.denom.clone(),
                        amount: total,
                    };
                    break;
                } else if info.funds.len() == 1 {
                    return Err(ContractError::InsufficientFee(total, coin.amount));
                }
            }
        }
    }
    

    Ok(())
}