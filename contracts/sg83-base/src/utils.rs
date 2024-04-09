use cosmwasm_std::{Coin, Storage, MessageInfo, ensure};
use cw_utils::PaymentError;
use sg_tba::RegistryParams;
use crate::{error::ContractError, state::SUDO_PARAMS};



pub fn fair_split(
    storage: &dyn Storage,
    info : &MessageInfo,
) -> Result<(Vec<Coin>, Vec<Coin>), ContractError> {
    ensure!(!info.funds.is_empty(), PaymentError::NoFunds {});

    let params = SUDO_PARAMS.load(storage)?;

    let mut fair_burn_funds = Vec::<Coin>::with_capacity(1);
    let mut acc_forwards_funds= Vec::<Coin>::with_capacity(info.funds.len());

    for coin in info.funds.iter() {

        if !fair_burn_funds.is_empty() {
            acc_forwards_funds.push(coin.clone());
            continue;
        }

        let fee_coin = params.creation_fees.iter().find(|c| c.denom == coin.denom);

        if let Some(fee_coin) = fee_coin {

            ensure!(fee_coin.amount <= coin.amount, ContractError::InsufficientFee(
                fee_coin.amount.u128(), 
                coin.amount.u128()
            ));

            fair_burn_funds.push(fee_coin.clone());

            let remaining = coin.amount.checked_sub(fee_coin.amount)?;

            if !remaining.is_zero() {
                acc_forwards_funds.push(Coin {
                    denom: fee_coin.denom.clone(),
                    amount: remaining
                });
            }
        } else {
            acc_forwards_funds.push(coin.clone());
        }
    }

    ensure!(!fair_burn_funds.is_empty(), ContractError::NoFeeTokens {});

    Ok((fair_burn_funds, acc_forwards_funds))

}


pub fn validate_params(
    params: &RegistryParams,
) -> Result<(), ContractError> {
    ensure!(!params.allowed_sg82_code_ids.is_empty(), ContractError::InvalidCodeIds {});
    ensure!(!params.creation_fees.is_empty(), ContractError::InvalidCreationFees {});
    ensure!(!params.creation_fees.iter().any(|c| c.amount.is_zero()), ContractError::InvalidCreationFees {});
    Ok(())
}