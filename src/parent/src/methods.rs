use candid::Principal;
use ic_cdk::query;
use ic_scalable_misc::{
    enums::filter_type::FilterType,
    models::{identifier_model::Identifier, paged_response_models::PagedResponse},
};

use shared::group_model::{GroupFilter, GroupResponse, GroupSort};

use super::store::ScalableData;

// Method used to get all the groups from the child canisters filtered, sorted and paged
// requires composite queries to be released to mainnet
// #[query(composite = true)]
#[query]
async fn get_groups(
    limit: usize,
    page: usize,
    filters: Vec<GroupFilter>,
    filter_type: FilterType,
    sort: GroupSort,
) -> PagedResponse<GroupResponse> {
    ScalableData::get_child_canister_data(limit, page, filters, filter_type, sort).await
}

#[query]
fn decode_identifier(identifier: Principal) -> (u64, String, String) {
    let (_id, _canister, _kind) = Identifier::decode(&identifier);
    (_id, _canister.to_string(), _kind)
}

#[query]
fn encode_identifier(id: u64, principal: Principal, kind: String) -> Result<Principal, String> {
    let identifier = Identifier::new(id, principal, kind);
    identifier.unwrap().encode()
}
