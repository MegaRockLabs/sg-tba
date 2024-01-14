use cosmwasm_schema::cw_serde;
use cosmwasm_std::{QuerierWrapper, StdResult, StdError, Addr, Coin, WasmMsg, to_json_binary};
use sg_std::Response;

#[cw_serde]
pub struct TokenInfo {
    /// Contract address of the collection
    pub collection: String,
    /// Token id
    pub id: String
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
        return Err(StdError::generic_err("Unauthorized"));
    }

    Ok(())
}


pub fn append_fair_burn_msg(
    fair_burn_addr: &Addr,
    funds: Vec<Coin>,
    recipient: Option<&Addr>,
    response: Response,
) -> Response {
    response.add_message(WasmMsg::Execute {
        contract_addr: fair_burn_addr.to_string(),
        msg: to_json_binary(&sg_fair_burn::msg::ExecuteMsg::FairBurn {
            recipient: recipient.map(|r| r.to_string()),
        })
        .unwrap(),
        funds,
    })
}