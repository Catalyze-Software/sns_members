# Member canister

This repository is responsible for handling group members of the Catalyze application. Members hold the users joined groups and outstanding invites.

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

- keeping track of all member child canisters
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

// Method used to get all the members from the child canisters filtered, sorted and paged
// requires composite queries to be released to mainnet
async fn get_members(
    group_identifier: Principal,
    limit: usize,
    page: usize,
) -> PagedResponse<JoinedMemberResponse> {}

// Method used to get all the members from the child canisters filtered, sorted and paged
// requires composite queries to be released to mainnet
async fn get_invites(
    group_identifier: Principal,
    limit: usize,
    page: usize,
) -> PagedResponse<InviteMemberResponse> {}

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
// Method to get the groups specific members are member of
fn get_groups_for_members(member_identifiers: Vec<Principal>) -> Vec<(Principal, Vec<Principal>)> {}

// Method to get the amount of invites of specific groups
fn get_group_invites_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {}

// Method to get all members of a specific group
fn get_group_members(group_identifier: Principal) -> Result<Vec<JoinedMemberResponse>, ApiError> {}

// Method to get the caller member entry
fn get_self() -> Result<(Principal, Member), ApiError> {}

// Get the roles of a specific member within a specific group
fn get_member_roles(
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Vec<String>), String> {}

// Method to fetch a specific group member by user principal
fn get_group_member(
    principal: Principal,
    group_identifier: Principal,
) -> Result<JoinedMemberResponse, ApiError> {}

// Method to get the amount of members of specific groups
fn get_group_members_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {}
```

###

###### UPDATE CALLS

```
// This method is used to join an existing group
async fn join_group(
    group_identifier: Principal,
    account_identifier: Option<String>,
) -> Result<(Principal, Member), ApiError> {}

// This method is used to create an empty member when a profile is created (inter-canister call)
async fn create_empty_member(
    caller: Principal,
    profile_identifier: Principal,
) -> Result<Principal, ApiError> {}

// This method is used to invite a user to a group
async fn invite_to_group(
    member_principal: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Member), ApiError> {}

// This method is used to accept an invite to a group as a admin
async fn accept_user_request_group_invite(
    member_principal: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Member), ApiError> {}

// This method is used to accept an invite to a group as a user
async fn accept_owner_request_group_invite(
    group_identifier: Principal,
) -> Result<(Principal, Member), ApiError> {}

// This method is used a to add an owner to the member entry when a group is created (inter-canister call)
async fn add_owner(
    owner_principal: Principal,
    group_identifier: Principal,
) -> Result<Principal, ApiError> {}

// Method to assign a role to a specific group member
async fn assign_role(
    role: String,
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(), ()> {}

// Method to remove a role from a specific group member
async fn remove_role(
    role: String,
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(), ()> {}

// Method to let the caller leave a group
fn leave_group(group_identifier: Principal) -> Result<(), ApiError> {}

// Method to remove an outstanding invite for a group as a user
fn remove_invite(group_identifier: Principal) -> Result<(), ApiError> {}

// Method to remove a member from a group
async fn remove_member_from_group(
    principal: Principal,
    group_identifier: Principal,
) -> Result<(), ApiError> {}

// Method to remove an outstanding invite for a group as a admin
async fn remove_member_invite_from_group(
    principal: Principal,
    group_identifier: Principal,
) -> Result<(), ApiError> {}

// Method to get all group invites
async fn get_group_invites(
    group_identifier: Principal,
) -> Result<Vec<InviteMemberResponse>, ApiError> {}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get members the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
fn get_chunked_join_data(
    group_identifier: Principal,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get members the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
fn get_chunked_invite_data(
    group_identifier: Principal,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {}
```

## SNS controlled

// TBD

## Testing

// TBD
