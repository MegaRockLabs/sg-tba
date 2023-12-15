use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, StdResult, Reply, StdError, to_json_binary,
};
use cw_ownable::{get_ownership, initialize_owner};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use sg_tba::{MigrateAccountMsg, TokenInfo};
use sg_std::Response;

use crate::{
    state::{REGISTRY_ADDRESS, TOKEN_INFO, PUBKEY, STATUS, MINT_CACHE, SERIAL}, 
    msg::{QueryMsg, InstantiateMsg, ExecuteMsg, Status}, 
    error::ContractError, 
    query::{can_execute, valid_signature, valid_signatures, known_tokens, assets, full_info}, 
    execute::{
        try_execute, 
        try_update_ownership, 
        try_update_known_tokens, 
        try_forget_tokens, 
        try_update_known_on_receive, 
        try_transfer_token, 
        try_send_token, 
        try_freeze, 
        try_unfreeze, 
        try_change_pubkey, 
        try_mint_token, 
        try_purging,
        MINT_REPLY_ID
    }, 
};

#[cfg(target_arch = "wasm32")]
use crate::utils::query_if_registry;

pub const CONTRACT_NAME: &str = "crates:sg82-token-account";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _ : Env, info : MessageInfo, msg : InstantiateMsg) 
-> Result<Response, ContractError> {

    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw22::set_contract_supported_interface(
        deps.storage, 
        &[
            cw22::ContractSupportedInterface {
                supported_interface: cw82::INTERFACE_NAME.into(),
                version: CONTRACT_VERSION.into()
            },
            cw22::ContractSupportedInterface {
                supported_interface: "crates:cw81".into(),
                version: CONTRACT_VERSION.into()
            },
            cw22::ContractSupportedInterface {
                supported_interface: "crates:cw1".into(),
                version: "1.1.1".into()
            },
        ]
    )?;

    #[cfg(target_arch = "wasm32")]
    if !query_if_registry(&deps.querier, info.sender.clone())? {
        return Err(ContractError::Unauthorized {})
    };

    initialize_owner(deps.storage, deps.api, Some(msg.owner.as_str()))?;
    
    TOKEN_INFO.save(deps.storage, &TokenInfo {
        collection: msg.token_contract,
        id: msg.token_id
    })?;

    REGISTRY_ADDRESS.save(deps.storage, &info.sender.to_string())?;
    STATUS.save(deps.storage, &Status { frozen: false })?;
    PUBKEY.save(deps.storage, &msg.pubkey)?;
    SERIAL.save(deps.storage, &0u128)?;

    Ok(Response::default()
)
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env : Env, info : MessageInfo, msg : ExecuteMsg) 
-> Result<Response, ContractError> {

    if !REGISTRY_ADDRESS.exists(deps.storage) {
        return Err(ContractError::Deleted {})
    }
    SERIAL.update(deps.storage, |s| Ok::<u128, StdError>(s + 1))?;

    match msg {
        ExecuteMsg::Execute { msgs } => try_execute(deps.as_ref(), info.sender, msgs),

        ExecuteMsg::MintToken { 
            minter: 
            collection, 
            msg 
        } => try_mint_token(deps, info.sender, collection, msg, info.funds),
        
        ExecuteMsg::TransferToken { 
            collection, 
            token_id, 
            recipient 
        } => try_transfer_token(deps, collection, token_id, recipient, info.funds),

        ExecuteMsg::SendToken { 
            collection, 
            token_id, 
            contract, 
            msg 
        } => try_send_token(deps, collection, token_id, contract, msg, info.funds),

        ExecuteMsg::UpdateKnownTokens { 
            collection, 
            start_after, 
            limit 
        } => try_update_known_tokens(
            deps, 
            env, 
            info.sender, 
            collection, 
            start_after, 
            limit
        ),

        ExecuteMsg::Freeze {} => try_freeze(deps, info.sender),
        
        ExecuteMsg::Unfreeze {} => try_unfreeze(deps),

        ExecuteMsg::ForgetTokens { 
            collection, 
            token_ids 
        } => try_forget_tokens(deps, info.sender, collection, token_ids),

        ExecuteMsg::ReceiveNft(
            msg
        ) => try_update_known_on_receive(deps, info.sender.to_string(), msg.token_id),
        
        ExecuteMsg::UpdateOwnership { 
            new_owner, 
            new_pubkey 
        } => try_update_ownership(deps, info.sender, new_owner, new_pubkey),

        ExecuteMsg::UpdatePubkey { new_pubkey } => try_change_pubkey(deps, info.sender, new_pubkey),

        ExecuteMsg::Purge {} => try_purging(deps, info.sender),
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env : Env, msg: QueryMsg) -> StdResult<Binary> {

    if !REGISTRY_ADDRESS.exists(deps.storage) {
        return Err(StdError::GenericErr { 
            msg: ContractError::Deleted {}.to_string() 
        })
    }

    match msg {
        QueryMsg::Token {} => to_json_binary(&TOKEN_INFO.load(deps.storage)?),
        QueryMsg::Status {} => to_json_binary(&STATUS.load(deps.storage)?),
        QueryMsg::Serial {} => to_json_binary(&SERIAL.load(deps.storage)?),
        QueryMsg::Pubkey {} => to_json_binary(&PUBKEY.load(deps.storage)?),
        QueryMsg::Registry {} => to_json_binary(&REGISTRY_ADDRESS.load(deps.storage)?),
        QueryMsg::Ownership {} => to_json_binary(&get_ownership(deps.storage)?),
        QueryMsg::CanExecute { 
            sender, 
            msg 
        } => to_json_binary(&can_execute(deps, sender, &msg)?),
        QueryMsg::ValidSignature { 
            signature, 
            data, 
            payload ,
        } => to_json_binary(&valid_signature(deps, data, signature, &payload)?),
        QueryMsg::ValidSignatures { 
            signatures, 
            data, 
            payload 
        } => to_json_binary(&valid_signatures(deps, data, signatures, &payload)?),
        QueryMsg::KnownTokens {
            skip,
            limit
        } => to_json_binary(&known_tokens(deps, skip, limit)?),
        QueryMsg::Assets {
            skip,
            limit
        } => to_json_binary(&assets(deps, env, skip, limit)?),
        QueryMsg::FullInfo {
            skip,
            limit
        } => to_json_binary(&full_info(deps, env, skip, limit)?)

    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _: Env, _: MigrateAccountMsg) -> StdResult<Response> {
    STATUS.save(deps.storage, &Status { frozen: false })?;
    Ok(Response::default())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        MINT_REPLY_ID => {
            let collection = MINT_CACHE.load(deps.storage)?;
            MINT_CACHE.remove(deps.storage);

            // query all the held tokens for the collection stored in CACHE
            try_update_known_tokens(
                deps, 
                env.clone(), 
                env.contract.address, 
                collection.to_string(), 
                None, 
                None
            )
        }

        _ => Err(ContractError::NotSupported {})
    }
}