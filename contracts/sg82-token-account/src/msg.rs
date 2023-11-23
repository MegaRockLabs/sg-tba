use cosmwasm_std::{Binary, Empty, CosmosMsg, Coin, Addr};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw721::Cw721ReceiveMsg;
pub use cw82::{
    smart_account_query, 
    CanExecuteResponse, 
    ValidSignatureResponse, 
    ValidSignaturesResponse
};
use cw_ownable::cw_ownable_query;


/// Instantiate message only callable by a cw83 registry. 
/// The contract uses the cw22 to check that the caller implements the interface
#[cw_serde]
pub struct InstantiateMsg {
    /// Token owner that had been verified by the registry
    pub owner: String,
    /// Public key used to verifiy signed messages
    pub pubkey: Binary,
    /// Contract address of the collection
    pub token_contract: String,
    /// Token id
    pub token_id: String
}


#[cw_serde]
pub struct TokenInfo {
    /// Contract address of the collection
    pub token_contract: String,
    /// Token id
    pub token_id: String
}


#[cw_serde]
pub struct PayloadInfo {
    /// Account address that public key corresponds to
    pub account: String,
    /// Algorithm to use for signature verification. Currently only "amino_direct" is supported
    pub algo: String
}


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
    }
}

/// [TokenInfo] is used as a to query the account info
/// so no need to return any additional data
pub type QueryMsg = QueryMsgBase<Empty>;


#[cw_serde]
pub enum ExecuteMsg {
    /// Proxy method for executing cosmos messages
    /// Wasm and Stargate messages aren't supported
    /// Only the current holder can execute this method
    Execute { 
        msgs: Vec<CosmosMsg<Empty>> 
    },
    /// Mint NFTs directly from token account
    MintToken { 
        /// Contract address of the minter
        minter: String, 
        // Mint message to pass a minter contract
        msg: Binary 
    },
    /// Send NFT to a contract
    SendToken { 
        /// Contract address of the collection
        collection: String, 
        /// Token id
        token_id: String, 
        /// Recipient contract address
        contract: String, 
        /// Send message to pass a recipient contract
        msg: Binary 
    },
    /// Simple NFT transfer
    TransferToken { 
        /// Contract address of the collection
        collection: String, 
        /// Token id
        token_id: String, 
        /// Recipient address
        recipient: String  
    },
    /// Owner only method to make the account forget about certain tokens
    ForgetTokens { 
        /// Contract address of the collection
        collection: String, 
        /// Optional list of token ids to forget. If not provided, all tokens will be forgotten
        token_ids: Vec<String> 
    },

    /// Owner only method that make the account aware of certain tokens to simplify the future queries
    UpdateKnownTokens { 
        /// Contract address of the collection
        collection: String, 
        /// Token id to start after
        start_after: Option<String>,
        /// Limit of the tokens to return 
        limit: Option<u32> 
    },

    /// Registry only method to update the owner to the current NFT holder
    UpdateOwnership { 
        /// Current NFT holder
        new_owner: String, 
        /// New secp256k1 public key
        new_pubkey: Binary 
    },

    /// Owner only method to update a public key
    UpdatePubkey { 
        /// New secp256k1 public key
        new_pubkey: Binary 
    },

    /// Registering a token as known on receiving
    ReceiveNft(Cw721ReceiveMsg),
    
    /// Registry only method to call when a token is moved to escrow
    Freeze {},

    /// Registry only method to call after the token is released from escrow
    Unfreeze {},
}


#[cw_serde]
pub struct MigrateMsg<T> {
    pub params: Option<Box<T>>
}