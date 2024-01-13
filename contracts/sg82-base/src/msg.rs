use cosmwasm_std::{Binary, Empty, Coin, Addr};
use cosmwasm_schema::{cw_serde, QueryResponses};
pub use cw82::{
    smart_account_query, 
    CanExecuteResponse, 
    ValidSignatureResponse, 
    ValidSignaturesResponse
};
use cw_ownable::cw_ownable_query;
use sg_std::StargazeMsgWrapper;
use sg_tba::{TokenInfo, InstantiateAccountMsg, ExecuteAccountMsg, MigrateAccountMsg};


pub type InstantiateMsg = InstantiateAccountMsg;
pub type MigrateMsg = MigrateAccountMsg;
pub type ExecuteMsg = ExecuteAccountMsg;



#[cw_serde]
pub struct Status {
    /// Whether the account is frozen
    pub frozen: bool,
}

#[cw_serde]
pub struct AssetsResponse {
    /// Native fungible tokens held by an account
    pub balances: Vec<Coin>,
    /// NFT tokens the account is aware of
    pub tokens: Vec<TokenInfo>
}



#[cw_serde]
pub struct FullInfoResponse {
    /// Current owner of the token account that is ideally a holder of an NFT
    pub ownership: cw_ownable::Ownership<Addr>,
    /// Public key that is used to verify signed messages
    pub pubkey: Binary,
    /// Token info
    pub token_info: TokenInfo,
    /// Registry address
    pub registry: String,
    /// Native fungible tokens held by an account
    pub balances: Vec<Coin>,
    /// NFT tokens the account is aware of
    pub tokens: Vec<TokenInfo>,
    /// Whether the account is frozen
    pub status: Status
}


pub type KnownTokensResponse = Vec<TokenInfo>;


#[smart_account_query]
#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsgBase <T = Empty> {

    /// Public key that is used to verify signed messages
    #[returns(Binary)]
    Pubkey {},

    /// Status of the account telling whether it iz frozen
    #[returns(Status)]
    Status {},

    /// NFT token the account is linked to
    #[returns(TokenInfo)]
    Token {},

    /// Registry address
    #[returns(String)]
    Registry {},

    /// List of the tokens the account is aware of
    #[returns(KnownTokensResponse)]
    KnownTokens {
        skip: Option<u32>,
        limit: Option<u32>
    },

    /// List of the assets (balances + tokens) the account is aware of
    #[returns(AssetsResponse)]
    Assets {
        skip: Option<u32>,
        limit: Option<u32>
    },

    /// Full info about the account
    #[returns(FullInfoResponse)]
    FullInfo {
        skip: Option<u32>,
        limit: Option<u32>
    },

    /// Incremental number telling wether a direct interaction with the account has occured
    #[returns(u128)]
    Serial {}
}

/// [TokenInfo] is used as a to query the account info
/// so no need to return any additional data
pub type QueryMsg = QueryMsgBase<StargazeMsgWrapper>;
