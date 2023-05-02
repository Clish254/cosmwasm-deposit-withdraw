#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, PoolBalance, CONFIG, POOLBALANCE};

const CONTRACT_NAME: &str = "crates.io:deposit-withdraw";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = msg.owner.unwrap_or(info.sender.to_string());
    let validated_owner = deps.api.addr_validate(&admin)?;
    let config = Config {
        owner: validated_owner.clone(),
        accepted_denoms: msg.accepted_denoms,
    };

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", validated_owner.to_string()))
}

pub fn sender_is_owner(deps: &DepsMut, sender: &str) -> StdResult<bool> {
    let cfg = CONFIG.load(deps.storage)?;
    let can = cfg.is_owner(&sender);
    Ok(can)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<BankMsg>, ContractError> {
    let is_admin = sender_is_owner(&deps, &info.sender.as_str())?;
    if !is_admin {
        return Err(ContractError::Unauthorized {});
    }
    match msg {
        ExecuteMsg::Deposit {} => execute_deposit(deps, env, info),
        ExecuteMsg::Withdraw { denoms, to_addr } => {
            execute_withdraw(deps, env, info, denoms, to_addr)
        }
    }
}

pub fn execute_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response<BankMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let funds_len = info.funds.len();

    if funds_len < 1 {
        return Err(ContractError::NoCoinsSent {});
    }
    if funds_len > 2 {
        return Err(ContractError::TooManyCoins {});
    }

    if funds_len < 2 {
        return Err(ContractError::OnlyTwoAllowed {});
    }
    let unaccepted_denoms: Vec<String> = info
        .funds
        .iter()
        .filter(|coin| !config.accepted_denoms.contains(&coin.denom))
        .map(|coin| coin.denom.clone())
        .collect();

    let unaccepted_denoms_len = unaccepted_denoms.len();
    if unaccepted_denoms_len >= 1 {
        return Err(ContractError::DenomsNotSupported {
            denoms: unaccepted_denoms,
        });
    }
    for coin in &info.funds {
        let denom_pool_balance = POOLBALANCE.may_load(deps.storage, coin.denom.clone())?;
        match denom_pool_balance {
            Some(denom_balance) => {
                let updated_denom_balance = PoolBalance {
                    amount: denom_balance.amount + coin.amount.u128(),
                };
                POOLBALANCE.save(deps.storage, coin.denom.clone(), &updated_denom_balance)?
            }
            None => {
                let new_denom_balance = PoolBalance {
                    amount: coin.amount.u128(),
                };
                POOLBALANCE.save(deps.storage, coin.denom.clone(), &new_denom_balance)?
            }
        }
    }

    Ok(Response::new().add_attribute("action", "deposit"))
}

pub fn execute_withdraw(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    denoms: [String; 2],
    to_addr: String,
) -> Result<Response<BankMsg>, ContractError> {
    let mut coins: Vec<Coin> = Vec::new();
    for denom in denoms {
        let denom_pool_balance = POOLBALANCE.may_load(deps.storage, denom.clone())?;
        match denom_pool_balance {
            Some(denom_balance) => {
                let coin = Coin {
                    denom,
                    amount: Uint128::from(denom_balance.amount),
                };
                coins.push(coin)
            }
            None => {
                return Err(ContractError::InsufficientBalance { denom });
            }
        }
    }
    let msg: CosmosMsg<BankMsg> = CosmosMsg::Bank(BankMsg::Send {
        to_address: to_addr,
        amount: coins,
    });
    Ok(Response::new()
        .add_attribute("action", "withdraw")
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate};
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, Coin, Uint128};

    pub const ADDR1: &str = "addr1";
    pub const ADDR3: &str = "addr3";

    pub const DENOM1: &str = "denom1";
    pub const DENOM2: &str = "denom2";

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &vec![]);

        let msg = InstantiateMsg {
            owner: None,
            accepted_denoms: [String::from(DENOM1), String::from(DENOM2)],
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![attr("action", "instantiate"), attr("owner", ADDR1)]
        )
    }

    #[test]
    fn test_execute_deposit_with_accepted_denoms() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &vec![]);

        let msg = InstantiateMsg {
            owner: None,
            accepted_denoms: [String::from(DENOM1), String::from(DENOM2)],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let amout1: u128 = 10;
        let amout2: u128 = 20;
        let coin1 = Coin {
            denom: String::from(DENOM1),
            amount: Uint128::from(amout1),
        };

        let coin2 = Coin {
            denom: String::from(DENOM2),
            amount: Uint128::from(amout2),
        };
        let info = mock_info(ADDR1, &vec![coin1, coin2]);
        let msg = ExecuteMsg::Deposit {};
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes, vec![attr("action", "deposit")])
    }

    #[test]
    fn test_execute_withdraw_with_accepted_denoms() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &vec![]);

        let msg = InstantiateMsg {
            owner: None,
            accepted_denoms: [String::from(DENOM1), String::from(DENOM2)],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let amout1: u128 = 10;
        let amout2: u128 = 20;
        let coin1 = Coin {
            denom: String::from(DENOM1),
            amount: Uint128::from(amout1),
        };

        let coin2 = Coin {
            denom: String::from(DENOM2),
            amount: Uint128::from(amout2),
        };
        let info = mock_info(ADDR1, &vec![coin1, coin2]);
        let msg = ExecuteMsg::Deposit {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Withdraw {
            denoms: [String::from(DENOM1), String::from(DENOM2)],
            to_addr: String::from(ADDR3),
        };

        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes[0], attr("action", "withdraw"))
    }
}
