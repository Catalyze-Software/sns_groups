# Group canister

This repository is responsible for handling groups of the Catalyze application. Groups are the base of the whole application.

## setup

The parent canister is SNS controlled, the child canisters are controlled by their parent. Upgrading the child canister is done through the parent canister as the (gzipped) child wasm is included in the parent canister.

When the parent canister is upgraded it checks if the child wasm has changed (currently it generates a new wasm hash every time you run the script). if changed it upgrades the child canisters automatically.

## Project structure

**|- candid**
Contains the candid files for the `parent` and `child` canister.

**|- frontend**
Contains all declarations that are needed for the frontend

**|- scripts**
Contains a single script that generates the following files for the parent and child canisters;

- candid files
- frontend declarations
- wasms (gzipped and regular)

**|- src/child**
Contains codebase related to the child canisters
**|- src/parent**
Contains codebase related to the child canisters
**|- src/shared**
Contains data used by both codebases

**|- wasm**
Contains

- child wasm
- child wasm (gzipped)
- parent wasm
- parent wasm (gzipped)

## Parent canister

The parent canister manages all underlying child canisters.

#### This canister is responsible for;

- keeping track of all group child canisters
- spinning up a new child canisters
- composite query call to the children (preperation)

#### methods

Described methods can be found below, for more details you can check out the code which is inline commented

###### DEFAULT

```
// Stores the data in stable storage before upgrading the canister.
pub fn pre_upgrade() {}

// Restores the data from stable- to heap storage after upgrading the canister.
pub fn post_upgrade() {}

// Init methods thats get triggered when the canister is installed
pub fn init() {}
```

##

###### QUERY CALLS

```
// Method to retrieve an available canister to write updates to
fn get_available_canister() -> Result<ScalableCanisterDetails, String> {}

// Method to retrieve all the canisters
fn get_canisters() -> Vec<ScalableCanisterDetails> {}

// Method to retrieve the latest wasm version of the child canister that is currently stored
fn get_latest_wasm_version() -> WasmVersion {}

// HTTP request handler (canister metrics are added to the response)
fn http_request(req: HttpRequest) -> HttpResponse {}

// Method used to get all the groups from the child canisters filtered, sorted and paged
// requires composite queries to be released to mainnet
async fn get_groups(
    limit: usize,
    page: usize,
    filters: Vec<GroupFilter>,
    filter_type: FilterType,
    sort: GroupSort,
) -> PagedResponse<GroupResponse> {}
```

##

###### UPDATE CALLS

```
// Method called by child canister once full (inter-canister call)
// can only be called by a child canister
async fn close_child_canister_and_spawn_sibling(
    last_entry_id: u64,
    entry: Vec<u8>
    ) -> Result<Principal, ApiError> {}

// Method to accept cycles when send to this canister
fn accept_cycles() -> u64 {}
```

## Child canister

The child canister is where the data is stored that the app uses.

This canister is responsible for;

- storing data records
- data validation
- messaging the parent to spin up a new sibling

#### methods

Described methods can be found below, for more details you can check out the code which is inline commented

###### DEFAULT

```
// Stores the data in stable storage before upgrading the canister.
pub fn pre_upgrade() {}

// Restores the data from stable- to heap storage after upgrading the canister.
pub fn post_upgrade() {}

// Init methods thats get triggered when the canister is installed
pub fn init(parent: Principal, name: String, identifier: usize) {}
```

##

###### QUERY CALLS

```
// This method is used to get a group from the canister
fn get_group(identifier: Principal) -> Result<GroupResponse, ApiError> {}

// This method is used to get groups filtered and sorted with pagination
fn get_groups(
    limit: usize,
    page: usize,
    filters: Vec<GroupFilter>,
    filter_type: FilterType,
    sort: GroupSort,
) -> Result<PagedResponse<GroupResponse>, ApiError> {}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get filtered groups the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
fn get_chunked_data(
    filters: Vec<GroupFilter>,
    filter_type: FilterType,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {}

// This method is used to get the owner and privacy of a group
// This is used for inter-canister calls to determine is a user can do a group specific action
fn get_group_owner_and_privacy(
    group_identifier: Principal,
) -> Result<(Principal, Privacy), ApiError> {}

// Get multiple groups by their identifiers
fn get_groups_by_id(group_identifiers: Vec<Principal>) -> Result<Vec<GroupResponse>, ApiError> {}

// This method is used to get all the roles of a group
fn get_group_roles(group_identifier: Principal) -> Vec<GroupRole> {
    Store::get_group_roles(group_identifier)
}
```

###

###### UPDATE CALLS

```
// This method is used to add a group to the canister,
// The method is async because it optionally creates a new canister is created
async fn add_group(
    post_group: PostGroup,
    member_canister: Principal,
    account_identifier: Option<String>,
) -> Result<GroupResponse, ApiError> {}

// This method is used to edit a group
async fn edit_group(
    group_identifier: Principal,
    update_group: UpdateGroup,
    member_identifier: Principal,
) -> Result<GroupResponse, ApiError> {}

// This method is used to (soft) delete a group
async fn delete_group(
    group_identifier: Principal,
    member_identifier: Principal,
) -> Result<Group, ApiError> {}

// This method is used to add a custom role to a group
async fn add_role(
    group_identifier: Principal,
    role_name: String,
    color: String,
    index: u64,
    member_identifier: Principal,
) -> Result<GroupRole, ApiError> {}

// This method is used to remove a custom role from a group
async fn remove_role(
    group_identifier: Principal,
    role_name: String,
    member_identifier: Principal,
) -> Result<bool, ApiError> {}

// This method is used to update the persmissions of a specific role
async fn edit_role_permissions(
    group_identifier: Principal,
    role_name: String,
    post_permissions: Vec<PostPermission>,
    member_identifier: Principal,
) -> Result<bool, ApiError> {}

// This method is used as an inter canister call to update the member count per canister
// Member count is used for backend filtering
pub fn update_member_count(
    group_identifier: Principal,
    member_canister: Principal,
    member_count: usize,
) -> Result<(), bool> {}

// This call get triggered when a new canister is spun up
// the data is passed along to the new canister as a byte array
async fn add_entry_by_parent(entry: Vec<u8>) -> Result<(), ApiError> {}

// Method to accept cycles when send to this canister
fn accept_cycles() -> u64 {}

// HTTP request handler, canister metrics are added to the response by default
fn http_request(req: HttpRequest) -> HttpResponse {}
```

## SNS controlled

// TBD

## Testing

// TBD
