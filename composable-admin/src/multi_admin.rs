pub use require_admin::require_multi_admin as require_admin;

use cosmwasm_std::{
    HumanAddr, CanonicalAddr, StdResult, Extern, ReadonlyStorage, Env,
    Api, Querier, Storage, from_slice, to_vec, StdError, HandleResponse,
    Binary, to_binary
};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;

const ADMINS_KEY: &[u8] = b"i801onL3kf";

pub fn multi_admin_handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: MultiAdminHandleMsg,
    handle: impl MultiAdminHandle,
) -> StdResult<HandleResponse> {
    match msg {
        MultiAdminHandleMsg::AddAdmins { addresses } => handle.add_admins(deps, env, addresses)
    }
}

pub fn multi_admin_query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: MultiAdminQueryMsg,
    query: impl MultiAdminQuery,
) -> StdResult<Binary> {
    match msg {
        MultiAdminQueryMsg::Admins => query.query_admins(deps)
    }
}

pub trait MultiAdminHandle {
    fn add_admins<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &mut Extern<S, A, Q>,
        env: Env,
        addresses: Vec<HumanAddr>,
    ) -> StdResult<HandleResponse> {
        assert_admin(deps, &env)?;
        save_admins(deps, &addresses)?;
    
        Ok(HandleResponse::default())
    }
}

pub trait MultiAdminQuery {
    fn query_admins<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>
    )-> StdResult<Binary> {
        let addresses = load_admins(deps)?;
    
        to_binary(&MultiAdminQueryResponse { 
            addresses
        })
    }
}

pub struct DefaultHandleImpl;

impl MultiAdminHandle for DefaultHandleImpl { }

pub struct DefaultQueryImpl;

impl MultiAdminQuery for DefaultQueryImpl { }

#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MultiAdminHandleMsg {
    AddAdmins {
        addresses: Vec<HumanAddr>,
    }
}

#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MultiAdminQueryMsg {
    Admins
}

#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct MultiAdminQueryResponse {
    pub addresses: Vec<HumanAddr>
}

pub fn save_admins<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    addresses: &Vec<HumanAddr>
) -> StdResult<()> {
    let mut admins: Vec<CanonicalAddr> = 
        load(&deps.storage, ADMINS_KEY).unwrap_or(vec![]);
    
    for address in addresses {
        let canonical = deps.api.canonical_address(address)?;
        admins.push(canonical);
    }

    deps.storage.set(ADMINS_KEY, &to_vec(&admins)?);

    Ok(())
}

pub fn load_admins<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Vec<HumanAddr>> {
    let admins: Vec<CanonicalAddr> = load(&deps.storage, ADMINS_KEY)?;
    let mut result = Vec::with_capacity(admins.len());

    for admin in admins {
        result.push(deps.api.human_address(&admin)?)
    }

    Ok(result)
}

pub fn assert_admin<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
) -> StdResult<()> {
    let admins = load_admins(deps)?;

    if admins.contains(&env.message.sender) {
        return Ok(());
    }

    Err(StdError::unauthorized())
}

fn load<T: DeserializeOwned, S: ReadonlyStorage>(storage: &S, key: &[u8]) -> StdResult<T> {
    let result = storage.get(key).ok_or_else(||
        StdError::SerializeErr { 
            source: "load".into(),
            msg: "key not found".into(),
            backtrace: None
        }
    )?;

    from_slice(&result)
}
