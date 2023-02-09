use candid::candid_method;
use ic_cdk::query;
use ic_scalable_misc::enums::filter_type::FilterType;

use crate::models::group_model::{GroupFilter, GroupResponse};

use super::store::ScalableData;

#[query(composite = true)]
#[candid_method(query)]
async fn get_all_data(filters: Vec<GroupFilter>, filter_type: FilterType) -> Vec<GroupResponse> {
    ScalableData::get_child_canister_data(filters, filter_type).await
}
