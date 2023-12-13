use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;


#[cw_serde]
pub struct MigrateAccountMsg<T = Empty> {
    pub params: Box<Option<T>>,
}


#[cw_serde]
pub struct TokenInfo {
    /// Contract address of the collection
    pub collection: String,
    /// Token id
    pub id: String
}
