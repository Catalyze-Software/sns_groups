import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Address {
  'street' : string,
  'country' : string,
  'city' : string,
  'postal_code' : string,
  'label' : string,
  'state_or_province' : string,
  'house_number' : string,
  'house_number_addition' : string,
}
export type ApiError = { 'SerializeError' : ErrorMessage } |
  { 'DeserializeError' : ErrorMessage } |
  { 'NotFound' : ErrorMessage } |
  { 'ValidationError' : Array<ValidationResponse> } |
  { 'CanisterAtCapacity' : ErrorMessage } |
  { 'UpdateRequired' : UpdateMessage } |
  { 'Unauthorized' : ErrorMessage } |
  { 'Unexpected' : ErrorMessage } |
  { 'BadRequest' : ErrorMessage };
export type Asset = { 'Url' : string } |
  { 'None' : null } |
  { 'CanisterStorage' : CanisterStorage };
export type CanisterStorage = { 'None' : null } |
  { 'Manifest' : Manifest } |
  { 'Chunk' : ChunkData };
export interface ChunkData {
  'chunk_id' : bigint,
  'canister' : Principal,
  'index' : bigint,
}
export interface DateRange { 'end_date' : bigint, 'start_date' : bigint }
export interface ErrorMessage {
  'tag' : string,
  'message' : string,
  'inputs' : [] | [Array<string>],
  'location' : string,
}
export type FilterType = { 'Or' : null } |
  { 'And' : null };
export type GatedType = { 'Neuron' : Array<NeuronGated> } |
  { 'Token' : Array<TokenGated> };
export interface Group {
  'updated_on' : bigint,
  'banner_image' : Asset,
  'owner' : Principal,
  'name' : string,
  'matrix_space_id' : string,
  'tags' : Uint32Array | number[],
  'description' : string,
  'created_by' : Principal,
  'created_on' : bigint,
  'website' : string,
  'privacy' : Privacy,
  'wallets' : Array<[Principal, string]>,
  'image' : Asset,
  'member_count' : Array<[Principal, bigint]>,
  'location' : Location,
  'roles' : Array<GroupRole>,
  'is_deleted' : boolean,
}
export type GroupFilter = { 'Tag' : number } |
  { 'UpdatedOn' : DateRange } |
  { 'MemberCount' : [bigint, bigint] } |
  { 'Name' : string } |
  { 'Identifiers' : Array<Principal> } |
  { 'Owner' : Principal } |
  { 'CreatedOn' : DateRange };
export interface GroupResponse {
  'updated_on' : bigint,
  'banner_image' : Asset,
  'owner' : Principal,
  'name' : string,
  'matrix_space_id' : string,
  'tags' : Uint32Array | number[],
  'description' : string,
  'created_by' : Principal,
  'created_on' : bigint,
  'website' : string,
  'privacy' : Privacy,
  'image' : Asset,
  'identifier' : Principal,
  'member_count' : bigint,
  'location' : Location,
  'roles' : Array<GroupRole>,
  'is_deleted' : boolean,
}
export interface GroupRole {
  'permissions' : Array<Permission>,
  'name' : string,
  'color' : string,
  'protected' : boolean,
  'index' : [] | [bigint],
}
export type GroupSort = { 'UpdatedOn' : SortDirection } |
  { 'MemberCount' : SortDirection } |
  { 'Name' : SortDirection } |
  { 'CreatedOn' : SortDirection };
export interface HttpHeader { 'value' : string, 'name' : string }
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
}
export interface HttpResponse {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export type Location = { 'None' : null } |
  { 'Digital' : string } |
  { 'Physical' : PhysicalLocation } |
  { 'MultiLocation' : MultiLocation };
export interface Manifest { 'entries' : Array<ChunkData> }
export interface MultiLocation {
  'physical' : PhysicalLocation,
  'digital' : string,
}
export interface NeuronGated {
  'governance_canister' : Principal,
  'name' : string,
  'description' : string,
  'ledger_canister' : Principal,
  'rules' : Array<NeuronGatedRules>,
}
export type NeuronGatedRules = { 'IsDisolving' : boolean } |
  { 'MinStake' : bigint } |
  { 'MinAge' : bigint } |
  { 'MinDissolveDelay' : bigint };
export interface PagedResponse {
  'total' : bigint,
  'data' : Array<GroupResponse>,
  'page' : bigint,
  'limit' : bigint,
  'number_of_pages' : bigint,
}
export interface Permission {
  'name' : string,
  'actions' : PermissionActions,
  'protected' : boolean,
}
export interface PermissionActions {
  'edit' : boolean,
  'read' : boolean,
  'delete' : boolean,
  'write' : boolean,
}
export interface PhysicalLocation {
  'longtitude' : number,
  'address' : Address,
  'lattitude' : number,
}
export interface PostGroup {
  'banner_image' : Asset,
  'name' : string,
  'matrix_space_id' : string,
  'tags' : Uint32Array | number[],
  'description' : string,
  'website' : string,
  'privacy' : Privacy,
  'image' : Asset,
  'location' : Location,
}
export interface PostPermission {
  'name' : string,
  'actions' : PermissionActions,
}
export type Privacy = { 'Gated' : GatedType } |
  { 'Private' : null } |
  { 'Public' : null } |
  { 'InviteOnly' : null };
export type Result = { 'Ok' : null } |
  { 'Err' : ApiError };
export type Result_1 = { 'Ok' : GroupResponse } |
  { 'Err' : ApiError };
export type Result_2 = { 'Ok' : GroupRole } |
  { 'Err' : ApiError };
export type Result_3 = { 'Ok' : Group } |
  { 'Err' : ApiError };
export type Result_4 = { 'Ok' : boolean } |
  { 'Err' : ApiError };
export type Result_5 = { 'Ok' : [Principal, Privacy] } |
  { 'Err' : ApiError };
export type Result_6 = { 'Ok' : PagedResponse } |
  { 'Err' : ApiError };
export type Result_7 = { 'Ok' : Array<GroupResponse> } |
  { 'Err' : ApiError };
export type Result_8 = { 'Ok' : null } |
  { 'Err' : boolean };
export type SortDirection = { 'Asc' : null } |
  { 'Desc' : null };
export interface TokenGated {
  'principal' : Principal,
  'name' : string,
  'description' : string,
  'amount' : bigint,
  'standard' : string,
}
export interface UpdateGroup {
  'banner_image' : Asset,
  'name' : string,
  'tags' : Uint32Array | number[],
  'description' : string,
  'website' : string,
  'privacy' : Privacy,
  'image' : Asset,
  'location' : Location,
}
export interface UpdateMessage {
  'canister_principal' : Principal,
  'message' : string,
}
export interface ValidationResponse { 'field' : string, 'message' : string }
export interface _SERVICE {
  '__get_candid_interface_tmp_hack' : ActorMethod<[], string>,
  'accept_cycles' : ActorMethod<[], bigint>,
  'add_entry_by_parent' : ActorMethod<[Uint8Array | number[]], Result>,
  'add_group' : ActorMethod<[PostGroup, Principal, [] | [string]], Result_1>,
  'add_role' : ActorMethod<
    [Principal, string, string, bigint, Principal],
    Result_2
  >,
  'backup_data' : ActorMethod<[], string>,
  'delete_group' : ActorMethod<[Principal, Principal], Result_3>,
  'download_chunk' : ActorMethod<[bigint], [bigint, Uint8Array | number[]]>,
  'edit_group' : ActorMethod<[Principal, UpdateGroup, Principal], Result_1>,
  'edit_role_permissions' : ActorMethod<
    [Principal, string, Array<PostPermission>, Principal],
    Result_4
  >,
  'get_chunked_data' : ActorMethod<
    [Array<GroupFilter>, FilterType, bigint, bigint],
    [Uint8Array | number[], [bigint, bigint]]
  >,
  'get_group' : ActorMethod<[Principal], Result_1>,
  'get_group_owner_and_privacy' : ActorMethod<[Principal], Result_5>,
  'get_group_roles' : ActorMethod<[Principal], Array<GroupRole>>,
  'get_groups' : ActorMethod<
    [bigint, bigint, Array<GroupFilter>, FilterType, GroupSort, boolean],
    Result_6
  >,
  'get_groups_by_id' : ActorMethod<[Array<Principal>], Result_7>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'migration_add_groups' : ActorMethod<[Array<[Principal, Group]>], undefined>,
  'remove_role' : ActorMethod<[Principal, string, Principal], Result_4>,
  'total_chunks' : ActorMethod<[], bigint>,
  'update_member_count' : ActorMethod<[Principal, Principal, bigint], Result_8>,
}
