use std::str::FromStr;
use super::chain::Chain;
use super::msg::ProxyInstantiateMsg;
use cosm_orc::orchestrator::cosm_orc::tokio_block;
use cosm_orc::orchestrator::error::{ProcessError, CosmwasmError};
use cosm_orc::orchestrator::{Coin as OrcCoin, ExecResponse, Address, ChainTxResponse, QueryResponse, Denom};
use cosm_orc::orchestrator::{InstantiateResponse, SigningKey};
use cosm_tome::chain::request::TxOptions;
use cosm_tome::modules::bank::model::SendRequest;
use cosmrs::crypto::secp256k1;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Empty, CosmosMsg, WasmMsg, Binary, to_json_binary, from_json, Coin, Addr};

use cw1::CanExecuteResponse;
use sg82_base::msg::QueryMsg;
use sg83_base::msg::{InstantiateMsg, CreateAccountMsg, FairBurnInfo};
use serde::Serialize;
use serde::de::DeserializeOwned;
use sg_std::StargazeMsgWrapper;
use sg_tba::{MigrateAccountMsg, TokenInfo};

// contract names used by cosm-orc to register stored code ids / instantiated addresses:
pub const COLLECTION_NAME   : &str = "sg721_base";
pub const REGISTRY_NAME     : &str = "sg83_base";
pub const ACOUNT_NAME       : &str = "sg82_base";
pub const PROXY_NAME        : &str = "cw1_whitelist";
pub const FAIR_BURN_NAME    : &str = "stargaze_fair_burn";

pub const MAX_TOKENS: u32 = 10_000;
pub const CREATION_FB_FEE: u128 = 100_000_000;
pub const MINT_PRICE: u128 = 100_000_000;


pub fn creation_fees_wasm (chain: &Chain) -> Vec<Coin> {
    vec![Coin {
        denom: chain.cfg.orc_cfg.chain_cfg.denom.clone(),
        amount: CREATION_FB_FEE.into(),
    }]
}

pub fn creation_fees(chain: &Chain) -> Vec<OrcCoin> {
    vec![OrcCoin {
        denom: Denom::from_str(&chain.cfg.orc_cfg.chain_cfg.denom).unwrap(),
        amount: CREATION_FB_FEE.into(),
    }]
}

pub fn instantiate_registry(
    chain: &mut Chain,
    creator_addr: String,
    key: &SigningKey,
    fair_burn_addr: String,
) -> Result<InstantiateResponse, ProcessError> {
    
    let account_id = chain.orc.contract_map.code_id(ACOUNT_NAME)?;

    chain.orc.instantiate(
        REGISTRY_NAME,
        "registry_instantiate",
        &InstantiateMsg {

            params: sg_tba::RegistryParams {
                allowed_sg82_code_ids: vec![account_id],
                creation_fees: creation_fees_wasm(&chain),
                managers: vec![],
                extension: Empty {}
            },
            fee_burn_info: FairBurnInfo {
                developer_addr: Addr::unchecked(creator_addr.clone()),
                fair_burn_addr: Addr::unchecked(fair_burn_addr),
            },
        },
        key,
        Some(creator_addr.parse().unwrap()),
        vec![],
    )
}


pub fn instantiate_fair_burn(
    chain: &mut Chain,
    admin: String,
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {

    chain.orc.instantiate(
        FAIR_BURN_NAME,
        "proxy_instantiate",
        &sg_fair_burn::msg::InstantiateMsg {
            fee_bps: 5000,
            fee_manager: admin.clone(),
        },
        key,
        Some(admin.parse().unwrap()),
        vec![],
    )
}


pub fn instantiate_proxy(
    chain: &mut Chain,
    admin: String,
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {

    chain.orc.instantiate(
        PROXY_NAME,
        "proxy_instantiate",
        &ProxyInstantiateMsg {
            admins: vec![admin.clone()],
            mutable: true,
        },
        key,
        Some(admin.parse().unwrap()),
        vec![],
    )
}

pub fn instantiate_collection(
    chain: &mut Chain,
    creator_addr: String,
    minter: String,
    nonce: Option<&str>,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {
    // let infos: Vec<(String, DeployInfo)> =  chain.cfg.orc_cfg.contract_deploy_info.clone().into_iter().collect();
    
    let code_id = chain.orc.contract_map.code_id(COLLECTION_NAME)?;

    let init_msg : CosmosMsg::<Empty> = CosmosMsg::Wasm(WasmMsg::Instantiate { 
        admin: Some(minter.clone().to_string()), 
        code_id, 
        msg: to_json_binary(&sg721::InstantiateMsg {
            name: "test".to_string() + nonce.unwrap_or_default(),
            symbol: "test".to_string() + nonce.unwrap_or_default(),
            minter: minter,
            collection_info: sg721::CollectionInfo {
                creator: creator_addr,
                description: "todo!()".into(),
                image: "https://example.com/image.png".into(),
                external_link: None,
                explicit_content: None,
                start_trading_time: None,
                royalty_info: None,
            }
        }).unwrap(),
        funds: vec![],
        label: "collection".to_string() + nonce.unwrap_or_default(),
    });

    chain.orc.execute(
        PROXY_NAME,
        "proxy_collection_instantiate",
        &cw1::Cw1ExecuteMsg::Execute { 
            msgs: vec![init_msg] 
        },
        key,
        vec![],
    )
}



pub fn mint_token(
    chain: &mut Chain,
    collection: String,
    token_id: String,
    owner: String,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {

    let mint_msg = sg721_base::ExecuteMsg::Mint { 
        token_id, 
        owner, 
        token_uri: None, 
        extension: None
    };

    chain.orc.execute(
        PROXY_NAME,
        "proxy_token_mint",
        &cw1::Cw1ExecuteMsg::<Empty>::Execute { 
            msgs: vec![WasmMsg::Execute { 
                contract_addr: collection, 
                msg: to_json_binary(&mint_msg).unwrap(), 
                funds: vec![]
            }.into()] 
        },
        key,
        vec![],
    )
}


pub fn send_token(
    chain: &mut Chain,
    token_id: String,
    recipient: String,
    msg: Binary,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {

    let send_msg = sg721_base::ExecuteMsg::SendNft { 
        contract: recipient, 
        token_id, 
        msg
    };

    chain.orc.execute(
        COLLECTION_NAME,
        "send_nft_acknowledge",
        &send_msg,
        key,
        vec![],
    )
}




pub fn create_token_account(
    chain: &mut Chain,
    token_contract: String,
    token_id: String,
    pubkey: Binary,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {

    let chain_id = chain.cfg.orc_cfg.chain_cfg.chain_id.clone();

    let init_msg = sg_tba::CreateAccountPayload { 
        token_info: TokenInfo {
            collection: token_contract,
            id: token_id,
        }, 
        account_data: pubkey,
        create_for: None, 
    };


    let code_id = chain.orc.contract_map.code_id(ACOUNT_NAME)?;

    chain.orc.execute(
        REGISTRY_NAME, 
        "registry_create_account", 
        &sg83_base::msg::ExecuteMsg::CreateAccount(
            CreateAccountMsg {
                code_id,
                chain_id,
                msg: init_msg,
            }
        ), 
        key, 
        creation_fees(&chain)
    )
}


pub fn reset_token_account(
    chain: &mut Chain,
    token_contract: String,
    token_id: String,
    pubkey: Binary,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {

    let chain_id = chain.cfg.orc_cfg.chain_cfg.chain_id.clone();

    let init_msg = sg_tba::CreateAccountPayload { 
        token_info: TokenInfo {
            collection: token_contract,
            id: token_id,
        }, 
        account_data: pubkey,
        create_for: None,
    };

    let code_id = chain.orc.contract_map.code_id(ACOUNT_NAME)?;

    chain.orc.execute(
        REGISTRY_NAME, 
        "registry_reset_account", 
        &sg83_base::msg::ExecuteMsg::ResetAccount(
            CreateAccountMsg {
                code_id,
                chain_id,
                msg: init_msg,
            }
        ), 
        key, 
        creation_fees(&chain)
    )
}



pub fn migrate_token_account(
    chain: &mut Chain,
    token_contract: String,
    token_id: String,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {

    let code_id = chain.orc.contract_map.code_id(ACOUNT_NAME)?;

    let migrate_msg = sg83_base::msg::ExecuteMsg::MigrateAccount { 
        token_info: TokenInfo {
            collection: token_contract,
            id: token_id,
        }, 
        new_code_id: code_id, 
        msg: MigrateAccountMsg { params: Box::new(None) }
    };


    chain.orc.execute(
        REGISTRY_NAME, 
        "registry_reset_account", 
        &migrate_msg, 
        key, 
        vec![]
    )
}


#[cw_serde]
pub struct FullSetupData {
    pub proxy: String,
    pub registry: String,
    pub collection: String,
    pub token_id: String,
    pub token_account: String,

    pub signer_mnemonic: String,
    pub public_key: Binary,

    pub user_address: String,
}


pub fn get_init_address(
    res: ChainTxResponse
) -> String {
    res
        .find_event_tags(
            "instantiate".to_string(), 
            "_contract_address".to_string()
        )[0].value.clone()
}


pub fn full_setup(
    chain: &mut Chain,
) -> Result<FullSetupData, ProcessError> {

    let _start_time = latest_block_time(chain).plus_seconds(60);


    let user: super::chain::SigningAccount = chain.cfg.users[0].clone();
    let user_address = user.account.address.clone();

    let res_fb = instantiate_fair_burn(chain, user_address.clone(), &user.key).unwrap();
    let fair_burn = get_init_address(res_fb.res);

    let reg_init = instantiate_registry(chain, user_address.clone(), &user.key, fair_burn).unwrap();
    
    let registry = get_init_address(reg_init.res);

    let proxy = instantiate_proxy(chain, user_address.clone(), &user.key).unwrap().address;
   
    let init_res = instantiate_collection(
        chain, 
        user.account.address.clone(), 
        proxy.clone().to_string(),
        None,
        &user.key
    ).unwrap();

    let collection  = get_init_address(init_res.res);
    chain.orc.contract_map.add_address(COLLECTION_NAME, collection.clone()).unwrap();
    
    let mint_res = mint_token(
        chain, 
        collection.clone(),
        "1".to_string(),
        user.account.address.clone(), 
        &user.key
    ).unwrap();


    let token_id = mint_res
                .res
                .find_event_tags(
                    "wasm".to_string(), 
                    "token_id".to_string()
                )[0].value.clone();
            

    let signing : secp256k1::SigningKey = user.key.clone().try_into().unwrap();
    let pubkey : Binary = signing.public_key().to_bytes().into();
    

    let create_res = create_token_account(
        chain, 
        collection.clone(),
        token_id.clone(),
        pubkey.clone(),
        &user.key
    ).unwrap();


    let token_account = get_init_address(create_res.res);
    chain.orc.contract_map.add_address(ACOUNT_NAME, token_account.clone()).unwrap();

    Ok(FullSetupData {
        proxy: proxy.to_string(),
        registry,
        collection,
        token_id,
        token_account,

        signer_mnemonic: user.account.mnemonic,
        public_key: pubkey,

        user_address
    })

}
 



pub fn wasm_query<S: Serialize>(
    chain: &mut Chain,
    address: &String,
    msg: &S
) -> Result<QueryResponse, CosmwasmError> {

    let res = tokio_block(async { 
        chain.orc.client.wasm_query(
            Address::from_str(&address)?,
            msg
        )
        .await }
    );

    res
}

pub fn wasm_query_typed<R, S> (
    chain: &mut Chain,
    address: &String,
    msg: &S
) -> Result<R, CosmwasmError> 
where S: Serialize,
      R: DeserializeOwned
{
    let res = tokio_block(async { 
        chain.orc.client.wasm_query(
            Address::from_str(&address)?,
            msg
        )
        .await }
    )?;


    let res : R = from_json(
        &res.res.data.unwrap()
    ).unwrap();

    Ok(res)
}



pub fn query_token_owner(
    chain: &mut Chain,
    collection: String,
    token_id: String,
) -> Result<cw721::OwnerOfResponse, CosmwasmError> {

    let res = wasm_query(
        chain,
        &collection,
        &cw721::Cw721QueryMsg::OwnerOf {
            token_id, include_expired: None
        }
    ).unwrap();

    let owner_res : cw721::OwnerOfResponse = from_json(
        &res.res.data.unwrap()
    ).unwrap();

    Ok(owner_res)
}
 


// gen_users will create `num_users` random SigningKeys
// and then transfer `init_balance` of funds to each of them.
pub fn gen_users(
    chain: &mut Chain,
    num_users: u32,
    init_balance: u128,
    denom: Option<&String>,
) -> Vec<SigningKey> {
    let prefix = &chain.cfg.orc_cfg.chain_cfg.prefix;
    let base_denom = &chain.cfg.orc_cfg.chain_cfg.denom;
    let from_user = &chain.cfg.users[1];

    let mut users = vec![];
    for n in 0..num_users {
        users.push(SigningKey::random_mnemonic(n.to_string()));
    }

    let mut reqs = vec![];
    for user in &users {
        let mut amounts = vec![OrcCoin {
            amount: init_balance,
            denom: base_denom.parse().unwrap(),
        }];
        // add extra denom if specified
        if let Some(denom) = denom {
            amounts.push(OrcCoin {
                amount: init_balance,
                denom: denom.parse().unwrap(),
            });
        }
        reqs.push(SendRequest {
            from: from_user.account.address.parse().unwrap(),
            to: user.to_addr(prefix).unwrap(),
            amounts,
        });
    }

    tokio_block(
        chain
            .orc
            .client
            .bank_send_batch(reqs, &from_user.key, &TxOptions::default()),
    )
    .unwrap();

    users
}

pub fn latest_block_time(chain: &Chain) -> Timestamp {
    let now = tokio_block(chain.orc.client.tendermint_query_latest_block())
        .unwrap()
        .block
        .header
        .unwrap()
        .time
        .unwrap();

    Timestamp::from_seconds(now.seconds.try_into().unwrap())
}



pub fn can_execute(
    chain: &mut Chain, 
    token_account: &String, 
    sender: String, 
    msg: CosmosMsg<StargazeMsgWrapper>
) -> CanExecuteResponse {
    let res = wasm_query(
        chain, 
        token_account, 
        &QueryMsg::CanExecute { 
            sender: sender, 
            msg: msg.into(), 
        }
    ).unwrap();
    
    from_json(
        &res.res.data.unwrap()
    ).unwrap()
}