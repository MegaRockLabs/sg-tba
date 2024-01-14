use cosmwasm_schema::{cw_serde, serde::Serialize};
use cosmwasm_std::{Empty, Binary, Coin};
use crate::common::TokenInfo;


#[cw_serde]
pub struct RegistryParams<T = Empty> {
    pub allowed_sg82_code_ids: Vec<u64>,
    pub creation_fees: Vec<Coin>,
    pub managers: Vec<String>,
    pub extension: T
}


/// An extenstion for [cw83::CreateAccountMsg]
#[cw_serde]
pub struct CreateAccountPayload<T = Binary> 
where T: Serialize
{
    /// Non-Fungible Token Info that the created account will be linked to 
    pub token_info: TokenInfo,

    /// Account data used for (cw81 signature verification)
    pub account_data: T,

    
    pub create_for: Option<String>
}
