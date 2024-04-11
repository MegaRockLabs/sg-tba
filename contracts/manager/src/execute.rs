use cosmwasm_std::{to_json_binary, DepsMut, Env, MessageInfo, WasmMsg };
use sg83_base::msg::{CreateAccountMsg, ExecuteMsg};
use sg_std::{CosmosMsg, Response};


use crate::{error::ContractError, payments::{process_ca_payment, process_wl_payment, shareholder_transfer_msgs}, state::{REGISTRY_CONTRACT, WL_COLLECTIONS}};


pub fn whitelist_collection(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    collection: String
) -> Result<Response, ContractError> {
    let amounts = process_wl_payment(deps.as_ref(), &info)?;

    let _msgs = shareholder_transfer_msgs(
        deps.storage,
        amounts.to_shareholders
    )?;
    
    WL_COLLECTIONS.save(deps.storage, &collection, &collection)?;
    Ok(Response::default())
}

pub fn create_account(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: CreateAccountMsg
) -> Result<Response, ContractError> {
    let amounts = process_ca_payment(deps.as_ref(), &info)?;

    let registry = REGISTRY_CONTRACT.load(deps.storage)?;

    let transfer_msgs = shareholder_transfer_msgs(
        deps.storage,
        amounts.to_shareholders
    )?;

    let ca_msg : CosmosMsg = WasmMsg::Execute { 
        contract_addr: registry.into(), 
        msg: to_json_binary(&ExecuteMsg::CreateAccount(msg))?, 
        funds: info.funds
    }.into();


    let mut msgs = Vec::with_capacity(transfer_msgs.len() + 2);
    msgs.extend(transfer_msgs.into_iter());
    msgs.push(ca_msg);
    
    Ok(Response::default().add_messages(msgs))
}