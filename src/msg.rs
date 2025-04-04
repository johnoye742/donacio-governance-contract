use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {}

#[derive(Debug, JsonSchema, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RawDetails {
    pub title: String,
    pub description: String,
    pub email: String,
    pub fullname: String,
    pub amount_to_be_raised: String,
    pub denom: String,
    pub image_url: String,
    pub code_id: u64
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateFundraiser {
        details: RawDetails
    },
    IssueNFT {
        user_addr: Addr,
        token_id: String,
        token_uri: String,
        nft_addr: String
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<Addr>)]
    GetFundraisers {}
}
