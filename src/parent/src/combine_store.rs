use candid::{CandidType, Deserialize, Principal};
use ic_scalable_misc::{
    enums::{api_error_type::ApiError, filter_type::FilterType},
    models::paged_response_models::PagedResponse,
};
use shared::group_model::{GroupFilter, GroupResponse, GroupSort};

#[derive(CandidType, Clone, Deserialize)]
pub struct CombineStore;

pub static MEMBERS_CANISTER: &str = "5rbtj-6aaaa-aaaap-abgjq-cai";
pub static GROUPS_CANISTER: &str = "7l53v-aqaaa-aaaap-abggq-cai";
pub static PROFILES_CANISTER: &str = "5ycyv-iiaaa-aaaap-abgia-cai";
pub static EVENTS_CANISTER: &str = "7z3mm-maaaa-aaaap-abgfq-cai";
pub static EVENT_ATTENDEES_CANISTER: &str = "65wd6-vaaaa-aaaap-abgdq-cai";

impl CombineStore {
    pub fn get_groups() {
        // event count
        // member joined / request
        // starred
        // labels
        // boosted
    }

    pub async fn get_groups_from_canister(
        limit: usize,
        page: usize,
        filters: Vec<GroupFilter>,
        filter_type: FilterType,
        sort: GroupSort,
        include_invite_only: bool,
    ) -> Option<PagedResponse<GroupResponse>> {
        let group_canister_id = Principal::from_text(GROUPS_CANISTER).unwrap();
        let result: Result<(Result<PagedResponse<GroupResponse>, ApiError>,), _> = ic_cdk::call(
            group_canister_id,
            "get_groups",
            (limit, page, filters, filter_type, sort, include_invite_only),
        )
        .await;

        match result {
            Ok((Ok(response),)) => Some(response),
            _ => None,
        }
    }

    pub fn get_event_count_for_groups() {}
}
