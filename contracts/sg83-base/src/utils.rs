use cosmwasm_std::{Coin, Storage, MessageInfo, ensure, Api};
use cw_utils::{must_pay, PaymentError};
use crate::{error::ContractError, state::SUDO_PARAMS, msg::RegistryParams};



pub fn fair_split(
    storage: &dyn Storage,
    info : &MessageInfo,
) -> Result<(Vec<Coin>, Vec<Coin>), ContractError> {
    ensure!(!info.funds.is_empty(), PaymentError::NoFunds {});

    let params = SUDO_PARAMS.load(storage)?;

    let mut fair_burn_funds = Vec::<Coin>::with_capacity(info.funds.len());
    let mut acc_forwards_funds= Vec::<Coin>::with_capacity(info.funds.len());
    let mut fee_coin_found = false;

    for coin in info.funds.iter() {
        let fee_coin = params.creation_fees.iter().find(|c| c.denom == coin.denom);

        if let Some(fee_coin) = fee_coin {
            fee_coin_found = true;

            let amount = must_pay(info, &fee_coin.denom)?;

            ensure!(amount >= fee_coin.amount, ContractError::InsufficientFee(
                fee_coin.amount.u128(), 
                amount.u128()
            ));

            fair_burn_funds.push(fee_coin.clone());

            if amount > fee_coin.amount {
                acc_forwards_funds.push(Coin {
                    denom: fee_coin.denom.clone(),
                    amount: amount - fee_coin.amount
                });
            }
        } else {
            acc_forwards_funds.push(coin.clone());
        }
    }

    ensure!(fee_coin_found, ContractError::NoFeeTokens {});

    Ok((fair_burn_funds, acc_forwards_funds))

}


pub fn validate_params(
    params: &RegistryParams,
    api: &dyn Api,
) -> Result<(), ContractError> {
    ensure!(!params.allowed_sg82_code_ids.is_empty(), ContractError::InvalidCodeIds {});
    ensure!(!params.creation_fees.is_empty(), ContractError::InvalidCreationFees {});
    ensure!(!params.creation_fees.iter().any(|c| c.amount.is_zero()), ContractError::InvalidCreationFees {});

    params.extension.is_ok(api)?;

    Ok(())
}