use cosmwasm_std::{
    Response, 
    Env, 
    Binary, 
    DepsMut, 
    Coin, 
    SubMsg, 
    ReplyOn, 
    WasmMsg, 
    CosmosMsg, 
    Addr,
    to_json_binary, 
};

use cw83::CREATE_ACCOUNT_REPLY_ID;
use sg82_token_account::{
    msg::{TokenInfo, MigrateMsg}, 
    utils::verify_nft_ownership
};

use crate::{
    state::{LAST_ATTEMPTING, TOKEN_ADDRESSES, ADMINS, ALLOWED_IDS},
    helpers::construct_label, 
    error::ContractError
};

pub fn create_account(
    deps: DepsMut,
    env: Env,
    sender: String,
    chain_id: String,
    code_id: u64,
    token_info: TokenInfo,
    pubkey: Binary,
    funds: Vec<Coin>,
    reset: bool
) -> Result<Response, ContractError> {

    if env.block.chain_id != chain_id {
        return Err(ContractError::InvalidChainId {})
    }

    if !ALLOWED_IDS.load(deps.storage)?.contains(&code_id) {
        return Err(ContractError::InvalidCodeId {})
    }

    verify_nft_ownership(&deps.querier, &sender, token_info.clone())?;


    let mut res = Response::default()
        .add_attributes(vec![
            ("action",  if reset { "reset_account" } else { "create_account" }),
            ("token_contract", token_info.collection.as_str()),
            ("token_id", token_info.id.as_str()),
            ("code_id", code_id.to_string().as_str()),
            ("chain_id", chain_id.as_str()),
            ("owner", sender.as_str())
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
            msg: to_json_binary(&sg82_token_account::msg::ExecuteMsg::Purge {})?,
            funds: vec![]
        });

        label = construct_label(&token_info, Some(env.block.height));

    } else {
        label = construct_label(&token_info, None);
    }

    LAST_ATTEMPTING.save(deps.storage, &token_info)?;

    let init_msg = sg82_token_account::msg::InstantiateMsg {
        owner: sender.clone(),
        token_contract: token_info.collection.clone(),
        token_id: token_info.id.clone(),
        pubkey
    };

    Ok(res
       .add_submessage(SubMsg {
            id: CREATE_ACCOUNT_REPLY_ID,
            msg: cosmwasm_std::CosmosMsg::Wasm(
                WasmMsg::Instantiate { 
                    admin: Some(env.contract.address.to_string()), 
                    code_id, 
                    msg: to_json_binary(&init_msg)?, 
                    label,
                    funds
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
    new_pubkey: Option<Binary>,
    funds: Vec<Coin>,
    update_for: Option<Addr>
) -> Result<Response, ContractError> {

    let is_admin = ADMINS.load(deps.storage)?.is_admin(sender.as_ref());
    let owner = update_for.unwrap_or(sender.clone());

    verify_nft_ownership(&deps.querier, owner.as_str(), token_info.clone())?;

    let contract_addr = TOKEN_ADDRESSES.load(
        deps.storage, 
        (token_info.collection.as_str(), token_info.id.as_str())
    )?;

    if owner != sender && !is_admin {
        return Err(ContractError::Unauthorized {})
    }

    let msg = sg82_token_account::msg::ExecuteMsg::UpdateOwnership { 
        new_owner: owner.to_string(), 
        new_pubkey
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
    msg: MigrateMsg
) -> Result<Response, ContractError> {

    if !ALLOWED_IDS.load(deps.storage)?.contains(&new_code_id) {
        return Err(ContractError::InvalidCodeId {});
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