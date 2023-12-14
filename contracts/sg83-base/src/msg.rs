use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Empty, Addr};
use cw83::{registry_query, registry_execute, 
    CreateAccountMsg as CreateAccountMsgBase,
    AccountQuery as AccountQueryBase,
    AccountInfoResponse as AccountInfoResponseBase,
};
use sg_tba::{MigrateAccountMsg, TokenInfo, RegistryParams};

#[cw_serde]
pub struct InstantiateMsg {

    pub params: RegistryParams
}


/// An extenstion for [cw83::CreateAccountMsg]
#[cw_serde]
pub struct CreateInitMsg {
    /// Non-Fungible Token Info that the created account will be linked to 
    pub token_info: TokenInfo,

    /// Public key of the account used for (cw81 signature verification)
    pub pubkey: Binary,
}

/// A List of the collections registered in the registry
#[cw_serde]
pub struct CollectionsResponse {
    /// Contract addresses of each collections
    pub collections: Vec<String>,

}

/// An full account stored in the registry
#[cw_serde]
pub struct Account {
    /// Contract address of the collection
    pub collection: String,
    /// Token id
    pub id: String,
    /// Address of the token-bound account
    pub address: String,
}

/// An entry without collection address 
#[cw_serde]
pub struct CollectionAccount {
    /// Token id
    pub id: String,
    /// Address of the token-bound account
    pub address: String,
}


#[cw_serde]
pub struct AccountsResponse {
    /// Total number of accounts in the registry
    pub total: u32,
    /// List of the accounts matching the query
    pub accounts: Vec<Account>
}

#[cw_serde]
pub struct CollectionAccountsResponse {
    /// Total number of accounts of a specific collection
    pub total: u32,
    /// List of the accounts matching the query
    pub accounts: Vec<CollectionAccount>
}

pub type AccountQuery = AccountQueryBase<TokenInfo>;
pub type AccountInfoResponse = AccountInfoResponseBase<Empty>;
pub type CreateAccountMsg = CreateAccountMsgBase<CreateInitMsg>;


#[registry_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {

    /// Query all accounts in the registry in descending order
    #[returns(AccountsResponse)]
    Accounts {
        /// Number of accounts to skip
        /// [NOTE]: Not same as `start_after`
        skip: Option<u32>,
        /// Limit how many accounts to return
        limit: Option<u32>,
    },

    /// Query accounts linked to a token of a specific collection in descending order
    #[returns(CollectionAccountsResponse)]
    CollectionAccounts {
        /// Contract address of the collection
        collection: String,
        /// Number of accounts to skip
        skip: Option<u32>,
        /// Limit how many accounts to return
        limit: Option<u32>,
    },

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


#[registry_execute]
#[cw_serde]
pub enum ExecuteMsg {

    /// Update the owner of a token-bound account
    UpdateAccountOwnership {
        /// Non-Fungible Token Info that the existing account is linked to
        token_info: TokenInfo,
        /// New public key of the account used for (cw81 signature verification)
        new_pubkey: Option<Binary>,
        /// Admin only parameter to update the account on behalf of another user that holds the token
        update_for: Option<Addr>,
    },

    /// Create a new token-bound account. Access the old one will be forever lost
    ResetAccount(CreateAccountMsg),

    /// Migrate an account to the newer code version if the code id is allowed
    MigrateAccount {
        /// Non-Fungible Token Info that the existing account is linked to
        token_info: TokenInfo,
        /// New code id to migrate the account to
        new_code_id: u64,
        /// Migration message to be passed to the account contract
        msg: MigrateAccountMsg
    }

}