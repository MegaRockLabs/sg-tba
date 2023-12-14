use cosmwasm_std::{
    Deps, DepsMut, Env, Addr, Binary, WasmMsg, Coin, StdResult, SubMsg, ReplyOn, to_json_binary,
};

use cw_ownable::{assert_owner, initialize_owner, is_owner};
use crate::{
    error::ContractError, 
    utils::{is_ok_cosmos_msg, assert_status, assert_registry, verify_nft_ownership}, 
    state::{KNOWN_TOKENS, PUBKEY, STATUS, MINT_CACHE, TOKEN_INFO, REGISTRY_ADDRESS, SERIAL}, 
    msg::{Status, StargazeResult, StargazeResponse, StarCosmosMsg}, 
};

pub const MINT_REPLY_ID: u64 = 1;


pub fn try_execute(
    deps: Deps,
    sender: Addr,
    msgs: Vec<StarCosmosMsg>
) -> StargazeResult {
    assert_owner(deps.storage, &sender)?;
    assert_status(deps.storage)?;
    if !msgs.iter().all(is_ok_cosmos_msg) {
        return Err(ContractError::NotSupported {})
    }
    Ok(StargazeResponse::new().add_messages(msgs))
}


pub fn try_mint_token(
    deps: DepsMut,
    sender: Addr,
    collection: String,
    msg: Binary,
    funds: Vec<Coin>
) -> StargazeResult {
    assert_owner(deps.storage, &sender)?;
    assert_status(deps.storage)?;

    MINT_CACHE.save(deps.storage, &collection)?;
    
    Ok(StargazeResponse::new().add_submessage(SubMsg {
        msg: WasmMsg::Execute { 
            contract_addr: collection, 
            msg, 
            funds
        }.into(),
        reply_on: ReplyOn::Success,
        id: MINT_REPLY_ID,
        gas_limit: None,
    }))
}


pub fn try_freeze(
    deps: DepsMut,
    sender: Addr
) -> StargazeResult {
    let token = TOKEN_INFO.load(deps.storage)?;
    let owner = cw_ownable::get_ownership(deps.storage)?.owner.unwrap();

    if owner != sender {

        // check if current owner still holds the token
        let verification = verify_nft_ownership(
            &deps.querier, 
            owner.as_str(), 
            token
        );

        if verification.is_ok() {
            // the token is not in escrow it isn't freezable by other entities
            return  Err(ContractError::Unauthorized {});
        }
    }

    STATUS.save(deps.storage, &Status { frozen: true })?;

    Ok(StargazeResponse::default()
        .add_attribute("action", "freeze"
    ))
}


pub fn try_unfreeze(
    deps: DepsMut,
) -> StargazeResult {

    let owner = cw_ownable::get_ownership(deps.storage)?.owner.unwrap();
    let token = TOKEN_INFO.load(deps.storage)?;
    
    verify_nft_ownership(
        &deps.querier, 
        owner.as_str(), 
        token
    )?;
    
    Ok(StargazeResponse::default()
        .add_attribute("action", "unfreeze"
    ))
}


pub fn try_update_ownership(
    deps: DepsMut,
    sender: Addr,
    new_owner: String,
    new_pubkey: Option<Binary>
) -> StargazeResult {
    assert_registry(deps.storage, &sender)?;
    initialize_owner(deps.storage, deps.api, Some(&new_owner))?;

    if new_pubkey.is_some() {
        PUBKEY.save(deps.storage, &new_pubkey.unwrap())?;
        STATUS.save(deps.storage, &Status { frozen: false })?;
    }

    Ok(
        StargazeResponse::default()
            .add_attribute("action", "update_ownership")
            .add_attribute("new_owner", new_owner.as_str())
    )
}


pub fn try_change_pubkey(
    deps: DepsMut,
    sender: Addr,
    pubkey: Binary
) -> StargazeResult {
    assert_owner(deps.storage, &sender)?;
    assert_status(deps.storage)?;
    PUBKEY.save(deps.storage, &pubkey)?;
    Ok(StargazeResponse::new()
        .add_attributes(vec![
            ("action", "change_pubkey"),
            ("new_pubkey", pubkey.to_base64().as_str())
        ])
    )
}



pub fn try_forget_tokens(
    deps: DepsMut,
    sender: Addr, 
    collection: String,
    token_ids: Vec<String>
) -> StargazeResult {
    assert_owner(deps.storage, &sender)?;
    assert_status(deps.storage)?;

    let ids = if token_ids.len() == 0 {
        KNOWN_TOKENS
        .prefix(collection.as_str())
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<String>>>()?

    } else {
        token_ids
    };

    for id in ids {
        KNOWN_TOKENS.remove(
            deps.storage, 
            (collection.as_str(), id.as_str()), 
        );
    }

    Ok(StargazeResponse::new().add_attribute("action", "forget_tokens"))
}



pub fn try_update_known_tokens(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    contract_addr: String,
    start_after: Option<String>,
    limit: Option<u32>
) -> StargazeResult {
    assert_status(deps.storage)?;
    if !is_owner(deps.storage, &sender)? 
        && env.contract.address != sender  {
        return Err(ContractError::Ownership(cw_ownable::OwnershipError::NotOwner))
    }

    let res : cw721::TokensResponse = deps.querier.query_wasm_smart(
        contract_addr.clone(), 
        &sg721_base::msg::QueryMsg::Tokens { 
        owner: env.contract.address.to_string(), 
        start_after, 
        limit 
    })?;

    for id in res.tokens {
        KNOWN_TOKENS.save(
            deps.storage, 
            (contract_addr.as_str(), id.as_str()),
            &true
        )?;
    }

    Ok(StargazeResponse::new().add_attribute("action", "update_known_tokens"))
}



pub fn try_update_known_on_receive(
    deps: DepsMut,
    collection: String,
    token_id: String,
) -> StargazeResult {
    
    KNOWN_TOKENS.save(
        deps.storage, 
        (collection.as_str(), token_id.as_str()),
        &true
    )?;

    Ok(
        StargazeResponse::default()
            .add_attribute("action", "update_known_on_receive")
    )
}


pub fn try_transfer_token(
    deps: DepsMut,
    collection: String,
    token_id: String,
    recipient: String,
    funds: Vec<Coin>
) -> StargazeResult {
    assert_status(deps.storage)?;
    
    KNOWN_TOKENS.remove(
        deps.storage, 
        (collection.as_str(), token_id.as_str()),
    );

    let msg : StarCosmosMsg = WasmMsg::Execute { 
        contract_addr: collection, 
        msg: to_json_binary(&sg721_base::ExecuteMsg::TransferNft { 
            recipient, 
            token_id, 
        })?, 
        funds
    }.into();

    Ok(StargazeResponse::default()
        .add_message(msg)
        .add_attribute("action", "transfer_token")
    )
}



pub fn try_send_token(
    deps: DepsMut,
    collection: String,
    token_id: String,
    contract: String,
    msg : Binary,
    funds: Vec<Coin>
) -> StargazeResult {
    assert_status(deps.storage)?;
    
    KNOWN_TOKENS.remove(
        deps.storage, 
        (collection.as_str(), token_id.as_str()),
    );

    let msg : StarCosmosMsg = WasmMsg::Execute { 
        contract_addr: collection, 
        msg: to_json_binary(&sg721_base::ExecuteMsg::SendNft { 
            contract, 
            token_id, 
            msg
        })?, 
        funds
    }.into();

    Ok(StargazeResponse::default()
        .add_message(msg)
        .add_attribute("action", "send_token")
    )
}



pub fn try_purging(
    deps: DepsMut,
    sender: Addr
) -> StargazeResult {
    assert_registry(deps.storage, &sender)?;
    
    KNOWN_TOKENS.clear(deps.storage);
    REGISTRY_ADDRESS.remove(deps.storage);
    MINT_CACHE.remove(deps.storage);
    TOKEN_INFO.remove(deps.storage);
    SERIAL.remove(deps.storage);
    PUBKEY.remove(deps.storage);
    STATUS.remove(deps.storage);

    Ok(StargazeResponse::default()
        .add_attribute("action", "purge")
    )
}
