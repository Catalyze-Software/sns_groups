use crate::store::ENTRIES;
use shared::group_model::Group;

#[ic_cdk::query(guard = "auth")]
fn read_groups_entries() -> Vec<(String, Group)> {
    ENTRIES.with(|entries| entries.borrow().iter().collect::<Vec<(String, Group)>>())
}

// GUARDS
const ALLOWED: [&str; 3] = [
    // sam candid ui
    "nvifv-62idm-izjcy-rvy63-7tqjz-mg2d7-jiw6m-soqvp-hdayh-mnqf5-yqe",
    // catalyze development
    "syzio-xu6ca-burmx-4afo2-ojpcw-e75j3-m67o5-s5bes-5vvsv-du3t4-wae",
    // proxy
    "bwm3m-wyaaa-aaaag-qdiua-cai",
];

fn auth() -> Result<(), String> {
    if ALLOWED.contains(&ic_cdk::caller().to_string().as_str()) {
        Ok(())
    } else {
        Err("Unauthorized".to_string())
    }
}
