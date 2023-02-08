import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type ApiError = { 'SerializeError' : ErrorMessage } |
  { 'DeserializeError' : ErrorMessage } |
  { 'NotFound' : ErrorMessage } |
  { 'ValidationError' : Array<ValidationResponse> } |
  { 'CanisterAtCapacity' : ErrorMessage } |
  { 'UpdateRequired' : UpdateMessage } |
  { 'Unauthorized' : ErrorMessage } |
  { 'Unexpected' : ErrorMessage } |
  { 'BadRequest' : ErrorMessage };
export type CanisterType = { 'Empty' : null } |
  { 'Foundation' : null } |
  { 'Custom' : null } |
  { 'ScalableChild' : null } |
  { 'Scalable' : null };
export interface ErrorMessage {
  'tag' : string,
  'message' : string,
  'inputs' : [] | [Array<string>],
  'location' : string,
}
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
export interface PagedResponse {
  'total' : bigint,
  'data' : Array<WhitelistEntry>,
  'page' : bigint,
  'limit' : bigint,
  'number_of_pages' : bigint,
}
export type Result = { 'Ok' : boolean } |
  { 'Err' : ApiError };
export type Result_1 = { 'Ok' : Principal } |
  { 'Err' : ApiError };
export type Result_2 = { 'Ok' : ScalableCanisterDetails } |
  { 'Err' : string };
export type Result_3 = { 'Ok' : ScalableMetaData } |
  { 'Err' : ApiError };
export type Result_4 = { 'Ok' : WasmDetails } |
  { 'Err' : ApiError };
export type Result_5 = { 'Ok' : PagedResponse } |
  { 'Err' : ApiError };
export type Result_6 = { 'Ok' : ScalableCanisterDetails } |
  { 'Err' : ApiError };
export interface ScalableCanisterDetails {
  'entry_range' : [bigint, [] | [bigint]],
  'principal' : Principal,
  'wasm_version' : WasmVersion,
  'is_available' : boolean,
  'canister_type' : CanisterType,
}
export interface ScalableMetaData {
  'updated_at' : bigint,
  'canister_count' : bigint,
  'owner' : Principal,
  'name' : string,
  'created_at' : bigint,
  'used_data' : bigint,
  'cycles' : bigint,
  'has_child_wasm' : boolean,
  'parent' : Principal,
}
export interface UpdateMessage {
  'canister_principal' : Principal,
  'message' : string,
}
export interface ValidationResponse { 'field' : string, 'message' : string }
export interface WasmDetails {
  'updated_at' : bigint,
  'wasm_version' : WasmVersion,
  'created_at' : bigint,
  'label' : string,
  'bytes' : Uint8Array | number[],
  'wasm_type' : CanisterType,
}
export type WasmVersion = { 'None' : null } |
  { 'Version' : bigint } |
  { 'Custom' : null };
export interface WhitelistEntry {
  'principal' : Principal,
  'rights' : WhitelistRights,
  'created_on' : bigint,
  'label' : string,
}
export type WhitelistRights = { 'Read' : null } |
  { 'ReadWrite' : null } |
  { 'Owner' : null };
export interface _SERVICE {
  '__get_candid_interface_tmp_hack' : ActorMethod<[], string>,
  'accept_cycles' : ActorMethod<[], bigint>,
  'add_to_whitelist' : ActorMethod<
    [string, Principal, WhitelistRights],
    Result
  >,
  'add_wasm' : ActorMethod<[string, Uint8Array | number[]], Result>,
  'change_name' : ActorMethod<[string], boolean>,
  'close_child_canister_and_spawn_sibling' : ActorMethod<
    [Principal, bigint, Uint8Array | number[], [] | [Principal]],
    Result_1
  >,
  'get_available_canister' : ActorMethod<[], Result_2>,
  'get_canisters' : ActorMethod<[], Array<ScalableCanisterDetails>>,
  'get_latest_wasm_version' : ActorMethod<[], WasmVersion>,
  'get_metadata' : ActorMethod<[], Result_3>,
  'get_wasms' : ActorMethod<[], Result_4>,
  'get_whitelist' : ActorMethod<[bigint, bigint], Result_5>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'initialize_first_child_canister' : ActorMethod<[], Result_1>,
  'reinstall_child_canister' : ActorMethod<[Principal], Result_1>,
  'remove_from_whitelist' : ActorMethod<[Principal], Result>,
  'sanity_check' : ActorMethod<[], string>,
  'upgrade_child_canister' : ActorMethod<[Principal], Result_6>,
}
