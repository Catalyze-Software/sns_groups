use std::{borrow::Cow, collections::HashMap};

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_scalable_misc::{
    enums::{
        asset_type::Asset, location_type::Location, privacy_type::Privacy, sort_type::SortDirection,
    },
    models::{date_models::DateRange, group_role::GroupRole},
    traits::stable_storage_trait::StableStorableTrait,
};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;

#[derive(Clone, CandidType, Serialize, Deserialize, Debug)]
pub struct Group {
    pub name: String,
    pub description: String,
    pub website: String,
    pub location: Location,
    pub privacy: Privacy,
    pub owner: Principal,
    pub created_by: Principal,
    pub matrix_space_id: String,
    pub image: Asset,
    pub banner_image: Asset,
    pub tags: Vec<u32>,
    pub privacy_gated_type_amount: Option<u64>,
    pub roles: Vec<GroupRole>,
    pub is_deleted: bool,
    pub member_count: HashMap<Principal, usize>,
    pub wallets: HashMap<Principal, String>,
    pub updated_on: u64,
    pub created_on: u64,
}

impl StableStorableTrait for Group {}

impl Storable for Group {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Default for Group {
    fn default() -> Self {
        Self {
            name: Default::default(),
            description: Default::default(),
            website: Default::default(),
            location: Default::default(),
            privacy: Default::default(),
            owner: Principal::anonymous(),
            created_by: Principal::anonymous(),
            matrix_space_id: Default::default(),
            image: Default::default(),
            banner_image: Default::default(),
            tags: Default::default(),
            member_count: Default::default(),
            wallets: Default::default(),
            roles: Vec::default(),
            is_deleted: Default::default(),
            updated_on: Default::default(),
            created_on: Default::default(),
            privacy_gated_type_amount: Default::default(),
        }
    }
}

#[derive(Clone, CandidType, Deserialize)]
pub struct PostGroup {
    pub name: String,
    pub description: String,
    pub website: String,
    pub matrix_space_id: String,
    pub location: Location,
    pub privacy: Privacy,
    pub privacy_gated_type_amount: Option<u64>,
    pub image: Asset,
    pub banner_image: Asset,
    pub tags: Vec<u32>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct UpdateGroup {
    pub name: String,
    pub description: String,
    pub website: String,
    pub location: Location,
    pub privacy: Privacy,
    pub image: Asset,
    pub banner_image: Asset,
    pub tags: Vec<u32>,
}

#[derive(Clone, CandidType, Serialize, Deserialize, Debug)]
pub struct GroupResponse {
    pub identifier: Principal,
    pub name: String,
    pub description: String,
    pub website: String,
    pub location: Location,
    pub privacy: Privacy,
    pub created_by: Principal,
    pub owner: Principal,
    pub matrix_space_id: String,
    pub image: Asset,
    pub banner_image: Asset,
    pub tags: Vec<u32>,
    pub roles: Vec<GroupRole>,
    pub member_count: usize,
    pub wallets: Vec<(Principal, String)>,
    pub is_deleted: bool,
    pub updated_on: u64,
    pub created_on: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum GroupSort {
    Name(SortDirection),
    MemberCount(SortDirection),
    CreatedOn(SortDirection),
    UpdatedOn(SortDirection),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum GroupFilter {
    Name(String),
    Owner(Principal),
    MemberCount((usize, usize)),
    Identifiers(Vec<Principal>),
    Tag(u32),
    UpdatedOn(DateRange),
    CreatedOn(DateRange),
}
