use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Empty, Coin};




#[cw_serde]
pub struct TokenInfo {
    /// Contract address of the collection
    pub collection: String,
    /// Token id
    pub id: String
}


#[cw_serde]
pub struct MigrateAccountMsg<T = Empty> {
    pub params: Box<Option<T>>,
}


#[cw_serde]
pub struct RegistryParams<T = Empty> {
    pub allowed_sg82_code_ids: Vec<u64>,
    pub creation_fee: Coin,
    pub managers: Vec<String>,
    pub extension: T
}
