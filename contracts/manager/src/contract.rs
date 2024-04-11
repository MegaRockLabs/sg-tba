use cosmwasm_std::{
    to_json_binary, DepsMut, Deps, Env, MessageInfo, StdResult, Binary
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use sg_std::Response;

use crate::{
    error::ContractError, execute, msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg}, query 
};

pub const CONTRACT_NAME: &str = "crates:sg83-tba-registry";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _ : Env, _ : MessageInfo, _ : InstantiateMsg) 
-> Result<Response, ContractError> {

    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw22::set_contract_supported_interface(
        deps.storage, 
        &[
            cw22::ContractSupportedInterface {
                supported_interface: cw83::INTERFACE_NAME.into(),
                version: CONTRACT_VERSION.into()
            }
        ]
    )?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "instantiate"),
        ])
    )
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env : Env, info : MessageInfo, msg : ExecuteMsg) 
-> Result<Response, ContractError> {

    match msg {
        ExecuteMsg::WhitelistCollection {
            collection
        } => execute::whitelist_collection(
            deps, 
            env,
            info,
            collection
        ),
        ExecuteMsg::CreateAccount(
            msg
        ) => execute::create_account(
            deps, 
            env,
            info,
            msg
        ),
    }

}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _ : Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Collections {
            skip,
            limit
        } => to_json_binary(&query::collections(deps, skip, limit)?),
    }
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default().add_attribute("action", "migrate"))
}