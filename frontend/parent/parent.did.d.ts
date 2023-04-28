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
export type CanisterType = { 'Empty' : null } |
  { 'Foundation' : null } |
  { 'Custom' : null } |
  { 'ScalableChild' : null } |
  { 'Scalable' : null };
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
export type Privacy = { 'Gated' : GatedType } |
  { 'Private' : null } |
  { 'Public' : null } |
  { 'InviteOnly' : null };
export type Result = { 'Ok' : Principal } |
  { 'Err' : ApiError };
export type Result_1 = { 'Ok' : ScalableCanisterDetails } |
  { 'Err' : string };
export interface ScalableCanisterDetails {
  'entry_range' : [bigint, [] | [bigint]],
  'principal' : Principal,
  'wasm_version' : WasmVersion,
  'is_available' : boolean,
  'canister_type' : CanisterType,
}
export type SortDirection = { 'Asc' : null } |
  { 'Desc' : null };
export interface TokenGated {
  'principal' : Principal,
  'name' : string,
  'description' : string,
  'amount' : bigint,
  'standard' : string,
}
export interface UpdateMessage {
  'canister_principal' : Principal,
  'message' : string,
}
export interface ValidationResponse { 'field' : string, 'message' : string }
export type WasmVersion = { 'None' : null } |
  { 'Version' : bigint } |
  { 'Custom' : null };
export interface _SERVICE {
  '__get_candid_interface_tmp_hack' : ActorMethod<[], string>,
  'accept_cycles' : ActorMethod<[], bigint>,
  'close_child_canister_and_spawn_sibling' : ActorMethod<
    [bigint, Uint8Array | number[]],
    Result
  >,
  'get_available_canister' : ActorMethod<[], Result_1>,
  'get_canisters' : ActorMethod<[], Array<ScalableCanisterDetails>>,
  'get_groups' : ActorMethod<
    [bigint, bigint, Array<GroupFilter>, FilterType, GroupSort],
    PagedResponse
  >,
  'get_latest_wasm_version' : ActorMethod<[], WasmVersion>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
}
