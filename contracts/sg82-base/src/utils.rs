use cosmwasm_std::{Addr, StdResult, Binary, StdError, CosmosMsg, WasmMsg, Storage, QuerierWrapper, from_json};
use sg_tba::TokenInfo;
use crate::{msg::{PayloadInfo, StarCosmosMsg}, error::ContractError, state::{STATUS, REGISTRY_ADDRESS}};

pub fn assert_status(
    store: &dyn Storage
) -> StdResult<bool>{
    let status = STATUS.load(store)?;
    Ok(!status.frozen)
}   

pub fn status_ok(
    store: &dyn Storage
) -> bool {
    assert_status(store).is_ok()
}


pub fn assert_ok_wasm_msg(
    msg: &WasmMsg
) -> StdResult<()> {
    let bad_wasm_error  = StdError::GenericErr { msg: "Not Supported".into() };
    match msg {
        // todo: add whitelististed messages
        WasmMsg::Execute { .. } => Err(bad_wasm_error),
        _ => Err(bad_wasm_error)
    }
}


pub fn assert_ok_cosmos_msg(
    msg: &StarCosmosMsg
) -> StdResult<()> {
    let bad_msg_error = StdError::GenericErr { msg: "Not Supported".into() };
    match msg {
        CosmosMsg::Wasm(msg) => assert_ok_wasm_msg(msg),
        CosmosMsg::Stargate { .. } => Err(bad_msg_error),
        _ => Ok(())
    }
}

pub fn is_ok_cosmos_msg(
    msg: &StarCosmosMsg
) -> bool {
    assert_ok_cosmos_msg(msg).is_ok()
}


pub fn query_if_registry(
    querier: &QuerierWrapper,
    addr: Addr
) -> StdResult<bool> {
    cw83::Cw83RegistryBase(addr).supports_interface(querier)
}



pub fn assert_registry(
    store: &dyn Storage,
    addr: &Addr
) -> Result<(), ContractError> {
    if is_registry(store, addr)? {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}


pub fn is_registry(
    store: &dyn Storage,
    addr: &Addr
) -> StdResult<bool> {
    REGISTRY_ADDRESS.load(store).map(|a| a == addr.to_string())
}


pub fn parse_payload(
    payload: &Option<Binary>
) -> StdResult<PayloadInfo> {

    if payload.is_none() {
        return Err(StdError::GenericErr { 
            msg: "Invalid payload. Must have an 'account' address and 'algo' must be 'amino_arbitrary'".into() 
        })
    }

    let payload : PayloadInfo = from_json(payload.as_ref().unwrap())?;
    
    if payload.account.len() < 1 || payload.algo != "amino_arbitrary" {
        return Err(StdError::GenericErr { 
            msg: "Invalid payload. Must have an 'account' address and 'amino_arbitrary' must be 'amino'".into() 
        })
    }

    Ok(payload)
}


pub fn generate_amino_transaction_string(signer: &str, data: &str) -> String {
    format!(
        "{{\"account_number\":\"0\",\"chain_id\":\"\",\"fee\":{{\"amount\":[],\"gas\":\"0\"}},\"memo\":\"\",\"msgs\":[{{\"type\":\"sign/MsgSignData\",\"value\":{{\"data\":\"{}\",\"signer\":\"{}\"}}}}],\"sequence\":\"0\"}}", 
        data, signer
    )
}


pub fn verify_nft_ownership(
    querier: &QuerierWrapper,
    sender: &str,
    token_info: TokenInfo
) -> StdResult<()> {

    let owner_res = querier
            .query_wasm_smart::<cw721::OwnerOfResponse>(
                token_info.collection, 
            &sg721_base::QueryMsg::OwnerOf {
                token_id: token_info.id,
                include_expired: None
            }
    )?;

    if owner_res.owner.as_str() != sender {
        return Err(StdError::GenericErr { msg: ContractError::Unauthorized {}.to_string() });
    }

    Ok(())
}
