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
export interface Invite {
  'updated_at' : bigint,
  'invite_type' : InviteType,
  'created_at' : bigint,
}
export interface InviteMemberResponse {
  'principal' : Principal,
  'group_identifier' : Principal,
  'invite' : Invite,
  'member_identifier' : Principal,
}
export type InviteType = { 'OwnerRequest' : null } |
  { 'UserRequest' : null };
export interface JoinedMemberResponse {
  'principal' : Principal,
  'group_identifier' : Principal,
  'member_identifier' : Principal,
  'roles' : Array<string>,
}
export interface PagedResponse {
  'total' : bigint,
  'data' : Array<InviteMemberResponse>,
  'page' : bigint,
  'limit' : bigint,
  'number_of_pages' : bigint,
}
export interface PagedResponse_1 {
  'total' : bigint,
  'data' : Array<JoinedMemberResponse>,
  'page' : bigint,
  'limit' : bigint,
  'number_of_pages' : bigint,
}
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
  'get_invites' : ActorMethod<[Principal, bigint, bigint], PagedResponse>,
  'get_latest_wasm_version' : ActorMethod<[], WasmVersion>,
  'get_members' : ActorMethod<[Principal, bigint, bigint], PagedResponse_1>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
}
