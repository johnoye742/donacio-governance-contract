#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, WasmMsg};
use cw721::msg::Cw721ExecuteMsg;
use donatio_crowdfund_contract::msg::InstantiateMsg as DonatioMsg;
use crate::msg::InstantiateMsg;
use crate::state::FUNDRAISERS;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, QueryMsg};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:donacio-governance";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/




#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateFundraiser { details } => {
            let instan = DonatioMsg {
                owner: info.sender,
                title: details.title,
                description: details.description,
                email: details.email,
                fullname: details.fullname,
                amount_to_be_raised: details.amount_to_be_raised,
                denom: details.denom,
                image_url: details.image_url
            };
            let msg = WasmMsg::Instantiate {
                admin: Some(String::from(env.contract.address.as_str())),
                code_id: details.code_id,
                msg: to_json_binary(&instan).unwrap(),
                funds: vec![],
                label: "donacio_fundraiser".to_string()
            };

            let submsg: SubMsg = SubMsg::reply_on_success(msg, 1);

            Ok(Response::new()
                .add_submessage(submsg))
        }

        ExecuteMsg::IssueNFT { user_addr, token_id, token_uri, nft_addr } => {
            let fundraisers: Vec<Addr> = FUNDRAISERS.load(deps.storage).unwrap();
            if fundraisers.contains(&info.sender) {
                let nft_msg: Cw721ExecuteMsg<_, String, Option<String>> = Cw721ExecuteMsg::Mint { token_id, owner: user_addr.into(), token_uri: Some(token_uri), extension: () };
                let msg = WasmMsg::Execute { contract_addr: nft_addr, msg: to_json_binary(&nft_msg)?, funds: vec![] };

                return Ok(Response::new()
                    .add_message(msg)
                    .add_attribute("action", "Issued NFT"))
            }
            Ok(Response::new())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetFundraisers {  } => {
            to_json_binary(&FUNDRAISERS.load(deps.storage).unwrap())
        }
    }
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    use cosmwasm_std::Event;

    if let Some(response) = msg.result.into_result().ok() {
        for event in response.events {
            if event.ty == "instantiate" {
                for attr in event.attributes {
                    if attr.key == "_contract_address" {
                        let contract_addr = Addr::unchecked(attr.value.clone());
                        let mut fundraisers = FUNDRAISERS.load(deps.storage).unwrap_or_default();
                        fundraisers.push(contract_addr);
                        FUNDRAISERS.save(deps.storage, &fundraisers)?;
                        return Ok(Response::new().add_attribute("stored_fundraiser", attr.value));
                    }
                }
            }
        }
    }

    Err(StdError::generic_err("Failed to retrieve contract address"))
}
#[cfg(test)]
mod tests {
    use crate::{contract, msg::QueryMsg};
    use cosmwasm_std::{Addr, Empty};
    use cw721::msg::Cw721InstantiateMsg;
    use cw_multi_test::{App, Contract, ContractWrapper, Executor, IntoAddr};

    use crate::msg::{ExecuteMsg, InstantiateMsg as Msg, RawDetails};


    fn create_contract() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(
            contract::execute,
            contract::instantiate,
            contract::query,
        ).with_reply(contract::reply))
    }

    fn donatio_crowdfund_contract() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(
            donatio_crowdfund_contract::contract::execute,
            donatio_crowdfund_contract::contract::instantiate,
            donatio_crowdfund_contract::contract::query,
        ))
    }

    fn donatio_nft_contract() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(
                donatio_nfts::execute,
                donatio_nfts::instantiate,
                donatio_nfts::query
        ))
    }

    #[test]
    pub fn can_create_fundraiser() {
        let mut app = App::default();

        let code_id = app.store_code(create_contract());

        let crowdfund_code_id = app.store_code(donatio_crowdfund_contract());

        let nft_code_id = app.store_code(donatio_nft_contract());


        let owner = "owner".into_addr();

        let admin = "admin".into_addr();

        let contract_addr = app.instantiate_contract(code_id, admin, &Msg {  }, &vec![], "donacio_governance", Some("admin".into_addr().as_str().into())).unwrap();

        let details = RawDetails {
            title: "Test Title".into(),
            description: "Test description".into(),
            email: "email@test.com".into(),
            fullname: "Test Name".into(),
            amount_to_be_raised: "200".into(),
            denom: "usdc".into(),
            image_url: "test_image.jpg".into(),
            code_id: crowdfund_code_id
        };

        let resp = app.execute_contract(owner.clone(),
            contract_addr.clone(),
            &ExecuteMsg::CreateFundraiser { details },
            &vec![]);

        let fundraisers: &Vec<Addr> = &app.wrap().query_wasm_smart(&contract_addr, &QueryMsg::GetFundraisers {  }).unwrap();

        let nft_contract = app.instantiate_contract(nft_code_id, contract_addr.clone(), &Cw721InstantiateMsg { name: "Donacio Collection".into(), symbol: "DNCC".into(), collection_info_extension: (), minter: Some(contract_addr.clone().into_string()), creator: Some(contract_addr.clone().into_string()), withdraw_address: Some("admin".into_addr().into_string()) }, &vec![], "donacio_nfts", Some(contract_addr.clone().into_string()));
        // GetAnNFTIssued
        &app.execute_contract(fundraisers[0].clone(), contract_addr, &ExecuteMsg::IssueNFT { user_addr: owner.into(), token_id: "d-1".into(), token_uri: "ipfs.io/".into(), nft_addr: nft_contract.unwrap().to_string() }, &vec![]);

        println!("Fundraisers: {:?}", &fundraisers);

        println!("{:?}", resp);
    }
}
