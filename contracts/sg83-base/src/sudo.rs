use cosmwasm_std::DepsMut;
use sg_std::Response;

use crate::{error::ContractError, state::SUDO_PARAMS, utils::validate_params, msg::RegistryParams};

/// Only governance can update contract params
pub fn sudo_update_params(
    deps: DepsMut,
    params: RegistryParams,
) -> Result<Response, ContractError> {
    validate_params(&params, deps.api)?;
    SUDO_PARAMS.save(deps.storage, &params)?;
    Ok(Response::new().add_attribute("action", "sudo_update_params"))
}
