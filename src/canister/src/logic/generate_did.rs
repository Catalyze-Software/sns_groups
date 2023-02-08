// Change imports accordingly
// When `export_service!()` gives a error, hover over it to see the missing imports

// can be trigger by running `cargo test --test generate`
// will write the file to `candid/canister.did`

#[test]
pub fn candid() {
    use crate::logic::default_methods::__export_did_tmp_;
    use ic_scalable_misc::helpers::candid_helper::save_candid;
    save_candid(__export_did_tmp_(), String::from("canister"));
}
