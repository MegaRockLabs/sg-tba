use cosmwasm_std::{DepsMut, StdResult, Coin};
use sg_std::Response;
use sg_tba::RegistryParams;

use crate::{error::ContractError, state::{SUDO_PARAMS, FAIR_BURN_INFO}, utils::validate_params, msg::FairBurnInfo};

/// Only governance can update contract params
pub fn sudo_update_params(
    deps: DepsMut,
    params: RegistryParams,
) -> Result<Response, ContractError> {
    validate_params(&params)?;
    SUDO_PARAMS.save(deps.storage, &params)?;
    Ok(Response::new().add_attribute("action", "sudo_update_params"))
}

pub fn sudo_update_allowed_sg82_code_ids(
    deps: DepsMut,
    ids: Vec<u64>,
) -> Result<Response, ContractError> {
    SUDO_PARAMS.update(deps.storage, |params| -> StdResult<_> {
        Ok(RegistryParams { 
            allowed_sg82_code_ids: ids,
            ..params
        })
    })?;
    Ok(Response::default()
        .add_attributes(vec![
            ("action", "sudo_update_allowed_sg82_code_ids"),
        ])
    )
}


pub fn sudo_update_creation_fees(
    deps: DepsMut,
    fees: Vec<Coin>,
) -> Result<Response, ContractError> {
    SUDO_PARAMS.update(deps.storage, |params| -> StdResult<_> {
        Ok(RegistryParams { 
            creation_fees: fees,
            ..params
        })
    })?;
    Ok(Response::default()
        .add_attributes(vec![
            ("action", "sudo_update_creation_fees"),
        ])
    )
}



pub fn sudo_update_managers(
    deps: DepsMut,
    managers: Vec<String>,
) -> Result<Response, ContractError> {
    SUDO_PARAMS.update(deps.storage, |params| -> StdResult<_> {
        Ok(RegistryParams { 
            managers,
            ..params
        })
    })?;
    Ok(Response::default()
        .add_attributes(vec![
            ("action", "sudo_update_managers"),
        ])
    )
}


pub fn sudo_update_fair_burn_address(
    deps: DepsMut,
    address: String,
) -> Result<Response, ContractError> {
    let fair_burn_addr = deps.api.addr_validate(address.as_str())?;
    FAIR_BURN_INFO.update(deps.storage, |info| -> StdResult<_> {
        Ok(FairBurnInfo { 
            fair_burn_addr,
            ..info
        })
    })?;
    Ok(Response::default()
        .add_attributes(vec![
            ("action", "update_fair_burn_address"),
            ("fair_burn_address", address.as_str())
        ])
    )
}