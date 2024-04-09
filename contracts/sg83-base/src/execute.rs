use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{
    Env, 
    Binary, 
    DepsMut, 
    Coin, 
    SubMsg, 
    ReplyOn, 
    WasmMsg, 
    Addr,
    MessageInfo, 
    to_json_binary, 
};

use cw83::CREATE_ACCOUNT_REPLY_ID;
use sg_std::{Response, CosmosMsg};
use sg_tba::{verify_nft_ownership, TokenInfo, 
    InstantiateAccountMsg, MigrateAccountMsg, ExecuteAccountMsg
};

type AccountMsg = ExecuteAccountMsg<Binary>;

use crate::{
    state::{LAST_ATTEMPTING, TOKEN_ADDRESSES, SUDO_PARAMS, FAIR_BURN_INFO},
    registry::construct_label, 
    error::ContractError, utils::fair_split
};

pub fn create_account<T: Serialize>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    chain_id: String,
    code_id: u64,
    token_info: TokenInfo,
    account_data: T,
    create_for: Option<String>,
    reset: bool,
) -> Result<Response, ContractError> {

    if env.block.chain_id != chain_id {
        return Err(ContractError::InvalidChainId {})
    }

    if !SUDO_PARAMS.load(deps.storage)?.allowed_sg82_code_ids.contains(&code_id) {
        return Err(ContractError::InvalidCodeId {})
    }

    let is_manager = SUDO_PARAMS.load(deps.storage)?.managers.contains(&info.sender.to_string());
    let owner = create_for.unwrap_or(info.sender.to_string());

    if owner != info.sender && !is_manager {
        return Err(ContractError::Unauthorized {})
    }

    verify_nft_ownership(&deps.querier, owner.as_str(), token_info.clone())?;

    let mut res = Response::default()
        .add_attributes(vec![
            ("action",  if reset { "reset_account" } else { "create_account" }),
            ("token_contract", token_info.collection.as_str()),
            ("token_id", token_info.id.as_str()),
            ("code_id", code_id.to_string().as_str()),
            ("chain_id", chain_id.as_str()),
            ("owner", info.sender.as_str())
        ]); 

    let token_address = TOKEN_ADDRESSES.may_load(
        deps.storage, 
        (token_info.collection.as_str(), token_info.id.as_str())
    )?;

    let label : String;

    if token_address.is_some() {

        if !reset {
            return Err(ContractError::AccountExists {})
        }

        res = res.add_message(WasmMsg::Execute {
            contract_addr: token_address.unwrap(),
            msg: to_json_binary(&AccountMsg::Purge {})?,
            funds: vec![]
        });

        label = construct_label(&token_info, Some(env.block.height));

    } else {
        label = construct_label(&token_info, None);
    }


    LAST_ATTEMPTING.save(deps.storage, &token_info)?;


    let init_msg = InstantiateAccountMsg {
        owner: info.sender.to_string(),
        token_info: token_info.clone(),
        account_data
    };


    let fb_info = FAIR_BURN_INFO.load(deps.storage)?;

    let (
        fair_burn_funds, 
        acc_forwards_funds
    ) = fair_split(deps.storage, &info)?;



    /* res = sg_tba::append_fair_burn_msg(
        &fb_info.fair_burn_addr, 
        fair_burn_funds, 
        Some(&fb_info.developer_addr), 
        res
    ); */


    Ok(res
       .add_submessage(SubMsg {
            id: CREATE_ACCOUNT_REPLY_ID,
            msg: cosmwasm_std::CosmosMsg::Wasm(
                WasmMsg::Instantiate { 
                    admin: Some(env.contract.address.to_string()), 
                    code_id, 
                    msg: to_json_binary(&init_msg)?, 
                    label,
                    funds: acc_forwards_funds
                }
            ),
            reply_on: ReplyOn::Success,
            gas_limit: None
        })
        
    )
}


pub fn update_account_owner(
    deps: DepsMut,
    sender: Addr,
    token_info: TokenInfo,
    new_account_data: Option<Binary>,
    funds: Vec<Coin>,
    update_for: Option<String>
) -> Result<Response, ContractError> {

    let is_manager = SUDO_PARAMS.load(deps.storage)?.managers.contains(&sender.to_string());
    let owner = update_for.unwrap_or(sender.to_string());
     // only admin can update ownership but only if the new address is the token owner
     if owner != sender && !is_manager {
        return Err(ContractError::Unauthorized {})
    }

    verify_nft_ownership(&deps.querier, owner.as_str(), token_info.clone())?;

    let contract_addr = TOKEN_ADDRESSES.load(
        deps.storage, 
        (token_info.collection.as_str(), token_info.id.as_str())
    )?;


    let msg = ExecuteAccountMsg::UpdateOwnership { 
        new_owner: owner.to_string(), 
        new_account_data
    };

    let msg = CosmosMsg::Wasm(WasmMsg::Execute { 
        contract_addr, 
        msg: to_json_binary(&msg)?, 
        funds 
    });

    Ok(Response::default()
        .add_message(msg)
        .add_attributes(vec![
            ("action", "update_account_owner"),
            ("token_contract", token_info.collection.as_str()),
            ("token_id", token_info.id.as_str()),
            ("new_owner", owner.to_string().as_str())
        ])
    )
}



pub fn migrate_account(
    deps: DepsMut,
    sender: Addr,
    token_info: TokenInfo,
    new_code_id: u64,
    msg: MigrateAccountMsg
) -> Result<Response, ContractError> {

    if !SUDO_PARAMS.load(deps.storage)?.allowed_sg82_code_ids.contains(&new_code_id) {
        return Err(ContractError::InvalidCodeId {})
    }

    verify_nft_ownership(&deps.querier, sender.as_str(), token_info.clone())?;

    let contract_addr = TOKEN_ADDRESSES.load(
        deps.storage, 
        (token_info.collection.as_str(), token_info.id.as_str())
    )?;

    let msg = CosmosMsg::Wasm(WasmMsg::Migrate { 
        contract_addr, 
        new_code_id, 
        msg: to_json_binary(&msg)?
    });
    

    Ok(Response::default()
        .add_message(msg)
         .add_attributes(vec![
            ("action", "migrate_account"),
            ("token_contract", token_info.collection.as_str()),
            ("token_id", token_info.id.as_str()),
            ("new_code_id", new_code_id.to_string().as_str())
          ])
    )
}