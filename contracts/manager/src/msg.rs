use cosmwasm_schema::{cw_serde, QueryResponses};
use sg83_base::msg::CreateAccountMsg;



#[cw_serde]
pub struct InstantiateMsg {}



/// A List of the collections registered in the registry
#[cw_serde]
pub struct CollectionsResponse {
    /// Total number of collections in the registry
    pub total: u32,
    /// Contract addresses of each collections
    pub collections: Vec<String>,

}




#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {

    /// Query all the collections the registry is aware of
    #[returns(CollectionsResponse)]
    Collections {
        /// Number of collections to skip
        skip: Option<u32>,
        /// Limit how many collections to return
        limit: Option<u32>
    },
}

#[cw_serde]
pub struct MigrateMsg {}


#[cw_serde]
pub enum ExecuteMsg {

    WhitelistCollection {
        /// Contract address of the collection
        collection: String,
    },

    CreateAccount(CreateAccountMsg)
}