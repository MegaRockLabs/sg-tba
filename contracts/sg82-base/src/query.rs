use cosmwasm_std::{StdResult, Deps, Binary, Order, Env, from_json};
use cw82::{CanExecuteResponse, ValidSignatureResponse, ValidSignaturesResponse};
use k256::sha2::{Digest, Sha256};
use cw_ownable::is_owner;
use sg_std::CosmosMsg;
use sg_tba::TokenInfo;

use crate::{
    state::{PUBKEY, KNOWN_TOKENS, TOKEN_INFO, STATUS, REGISTRY_ADDRESS}, 
    utils::{generate_amino_transaction_string, is_ok_cosmos_msg, status_ok, assert_status}, 
    msg::{AssetsResponse, FullInfoResponse}
};


const DEFAULT_BATCH_SIZE : u32 = 100;


pub fn can_execute(
    deps: Deps,
    sender: String,
    msg: &CosmosMsg
) -> StdResult<CanExecuteResponse> {

    let cant = CanExecuteResponse { can_execute: false };

    if !status_ok(deps.storage) { return Ok(cant) };

    let addr_validity = deps.api.addr_validate(&sender);
    if addr_validity.is_err() { return Ok(cant) };

    let res = is_owner(deps.storage, &addr_validity.unwrap());
    if res.is_err() || res.unwrap() == false { return Ok(cant) };

    Ok(CanExecuteResponse { can_execute: is_ok_cosmos_msg(msg) })
}


pub fn valid_signature(
    deps: Deps,
    data: Binary,
    signature: Binary,
    payload: &Option<Binary>
) -> StdResult<ValidSignatureResponse> {
    let pk: Binary = PUBKEY.load(deps.storage)?;
    let owner = cw_ownable::get_ownership(deps.storage)?;

    let address = match payload {
        Some(payload) => from_json(payload)?,
        None => owner.owner.unwrap().to_string()
    };

    Ok(ValidSignatureResponse {
        is_valid: match assert_status(deps.storage)? {
            true => verify_arbitrary(
                deps,
                &address,
                data,
                signature,
                &pk
            )?,
            false => false
        }
    })
}


pub fn valid_signatures(
    deps: Deps,
    data: Vec<Binary>,
    signatures: Vec<Binary>,
    payload: &Option<Binary>
) -> StdResult<ValidSignaturesResponse> {

    let status_ok = assert_status(deps.storage)?;

    let pk: Binary = PUBKEY.load(deps.storage)?;
    let owner = cw_ownable::get_ownership(deps.storage)?;

    let address = match payload {
        Some(payload) => from_json(payload)?,
        None => owner.owner.unwrap().to_string()
    };

    let are_valid : Vec<bool> = signatures
        .into_iter()
        .enumerate()
        .map(|(i, signature)| {
            if !status_ok { return false };
            let data = data.get(i).unwrap().clone();
            verify_arbitrary(
                deps,
                &address,
                data,
                signature,
                &pk
            ).unwrap_or(false)
        })
        .collect(); 
    
    Ok(ValidSignaturesResponse {
        are_valid
    })
}


pub fn verify_arbitrary(
    deps: Deps,
    account_addr: &str,
    data: Binary,
    signature: Binary,
    pubkey: &[u8],
) -> StdResult<bool> {

    let digest = Sha256::new_with_prefix(
        generate_amino_transaction_string(
        account_addr,
        from_json::<String>(&data)?.as_str(),
    )).finalize();

    deps.api.secp256k1_verify(
        &digest, 
        &signature, 
        pubkey
    )?;

    Ok(true)
}


pub fn assets(
    deps: Deps,
    env: Env,
    skip: Option<u32>,
    limit: Option<u32>
) -> StdResult<AssetsResponse> {

    let nfts = known_tokens(deps, skip, limit)?;
    let balance = deps.querier.query_all_balances(env.contract.address)?;

    Ok(AssetsResponse {
        balances: balance,
        tokens: nfts
    })
}


pub fn known_tokens(
    deps: Deps,
    skip: Option<u32>,
    limit: Option<u32>
) -> StdResult<Vec<TokenInfo>> {

    let skip  = skip.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(DEFAULT_BATCH_SIZE) as usize;

    let tokens : StdResult<Vec<TokenInfo>> = KNOWN_TOKENS
    .keys(
        deps.storage, 
        None, 
        None, 
        Order::Ascending
    )
    .enumerate()
    .filter(|(i, _)| *i >= skip)
    .take(limit)
    .map(|(_, kt)| {
        let kp = kt?;
        Ok(TokenInfo { collection: kp.0, id: kp.1 })
    })
    .collect();

    tokens
}


pub fn full_info(
    deps: Deps,
    env: Env,
    skip: Option<u32>,
    limit: Option<u32>
) -> StdResult<FullInfoResponse> {

    let tokens = known_tokens(deps, skip, limit)?;
    let balances = deps.querier.query_all_balances(env.contract.address)?;
    let ownership = cw_ownable::get_ownership(deps.storage)?;

    Ok(FullInfoResponse {
        balances,
        tokens,
        ownership,
        registry:   REGISTRY_ADDRESS.load(deps.storage)?,
        pubkey:     PUBKEY.load(deps.storage)?,
        token_info: TOKEN_INFO.load(deps.storage)?,
        status:     STATUS.load(deps.storage)?
    })
}