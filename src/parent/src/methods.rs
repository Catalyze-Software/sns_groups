use candid::candid_method;
use ic_cdk::query;
use ic_scalable_misc::{
    enums::filter_type::FilterType, models::paged_response_models::PagedResponse,
};

use shared::group_model::{GroupFilter, GroupResponse, GroupSort};

use super::store::ScalableData;

// Method used to get all the groups from the child canisters filtered, sorted and paged
// requires composite queries to be released to mainnet
#[query(composite = true)]
#[candid_method(query)]
async fn get_groups(
    limit: usize,
    page: usize,
    filters: Vec<GroupFilter>,
    filter_type: FilterType,
    sort: GroupSort,
) -> PagedResponse<GroupResponse> {
    ScalableData::get_child_canister_data(limit, page, filters, filter_type, sort).await
}
