export const idlFactory = ({ IDL }) => {
  const ErrorMessage = IDL.Record({
    'tag' : IDL.Text,
    'message' : IDL.Text,
    'inputs' : IDL.Opt(IDL.Vec(IDL.Text)),
    'location' : IDL.Text,
  });
  const ValidationResponse = IDL.Record({
    'field' : IDL.Text,
    'message' : IDL.Text,
  });
  const UpdateMessage = IDL.Record({
    'canister_principal' : IDL.Principal,
    'message' : IDL.Text,
  });
  const ApiError = IDL.Variant({
    'SerializeError' : ErrorMessage,
    'DeserializeError' : ErrorMessage,
    'NotFound' : ErrorMessage,
    'ValidationError' : IDL.Vec(ValidationResponse),
    'CanisterAtCapacity' : ErrorMessage,
    'UpdateRequired' : UpdateMessage,
    'Unauthorized' : ErrorMessage,
    'Unexpected' : ErrorMessage,
    'BadRequest' : ErrorMessage,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Principal, 'Err' : ApiError });
  const WasmVersion = IDL.Variant({
    'None' : IDL.Null,
    'Version' : IDL.Nat64,
    'Custom' : IDL.Null,
  });
  const CanisterType = IDL.Variant({
    'Empty' : IDL.Null,
    'Foundation' : IDL.Null,
    'Custom' : IDL.Null,
    'ScalableChild' : IDL.Null,
    'Scalable' : IDL.Null,
  });
  const ScalableCanisterDetails = IDL.Record({
    'entry_range' : IDL.Tuple(IDL.Nat64, IDL.Opt(IDL.Nat64)),
    'principal' : IDL.Principal,
    'wasm_version' : WasmVersion,
    'is_available' : IDL.Bool,
    'canister_type' : CanisterType,
  });
  const Result_1 = IDL.Variant({
    'Ok' : ScalableCanisterDetails,
    'Err' : IDL.Text,
  });
  const DateRange = IDL.Record({
    'end_date' : IDL.Nat64,
    'start_date' : IDL.Nat64,
  });
  const GroupFilter = IDL.Variant({
    'Tag' : IDL.Nat32,
    'UpdatedOn' : DateRange,
    'MemberCount' : IDL.Tuple(IDL.Nat64, IDL.Nat64),
    'Name' : IDL.Text,
    'Identifiers' : IDL.Vec(IDL.Principal),
    'Owner' : IDL.Principal,
    'CreatedOn' : DateRange,
  });
  const FilterType = IDL.Variant({ 'Or' : IDL.Null, 'And' : IDL.Null });
  const SortDirection = IDL.Variant({ 'Asc' : IDL.Null, 'Desc' : IDL.Null });
  const GroupSort = IDL.Variant({
    'UpdatedOn' : SortDirection,
    'MemberCount' : SortDirection,
    'Name' : SortDirection,
    'CreatedOn' : SortDirection,
  });
  const ChunkData = IDL.Record({
    'chunk_id' : IDL.Nat64,
    'canister' : IDL.Principal,
    'index' : IDL.Nat64,
  });
  const Manifest = IDL.Record({ 'entries' : IDL.Vec(ChunkData) });
  const CanisterStorage = IDL.Variant({
    'None' : IDL.Null,
    'Manifest' : Manifest,
    'Chunk' : ChunkData,
  });
  const Asset = IDL.Variant({
    'Url' : IDL.Text,
    'None' : IDL.Null,
    'CanisterStorage' : CanisterStorage,
  });
  const NeuronGatedRules = IDL.Variant({
    'IsDisolving' : IDL.Bool,
    'MinStake' : IDL.Nat64,
    'MinAge' : IDL.Nat64,
    'MinDissolveDelay' : IDL.Nat64,
  });
  const NeuronGated = IDL.Record({
    'governance_canister' : IDL.Principal,
    'name' : IDL.Text,
    'description' : IDL.Text,
    'rules' : IDL.Vec(NeuronGatedRules),
  });
  const TokenGated = IDL.Record({
    'principal' : IDL.Principal,
    'name' : IDL.Text,
    'description' : IDL.Text,
    'amount' : IDL.Nat64,
    'standard' : IDL.Text,
  });
  const GatedType = IDL.Variant({
    'Neuron' : IDL.Vec(NeuronGated),
    'Token' : IDL.Vec(TokenGated),
  });
  const Privacy = IDL.Variant({
    'Gated' : GatedType,
    'Private' : IDL.Null,
    'Public' : IDL.Null,
    'InviteOnly' : IDL.Null,
  });
  const Address = IDL.Record({
    'street' : IDL.Text,
    'country' : IDL.Text,
    'city' : IDL.Text,
    'postal_code' : IDL.Text,
    'label' : IDL.Text,
    'state_or_province' : IDL.Text,
    'house_number' : IDL.Text,
    'house_number_addition' : IDL.Text,
  });
  const PhysicalLocation = IDL.Record({
    'longtitude' : IDL.Float32,
    'address' : Address,
    'lattitude' : IDL.Float32,
  });
  const Location = IDL.Variant({
    'None' : IDL.Null,
    'Digital' : IDL.Text,
    'Physical' : PhysicalLocation,
  });
  const PermissionActions = IDL.Record({
    'edit' : IDL.Bool,
    'read' : IDL.Bool,
    'delete' : IDL.Bool,
    'write' : IDL.Bool,
  });
  const Permission = IDL.Record({
    'name' : IDL.Text,
    'actions' : PermissionActions,
    'protected' : IDL.Bool,
  });
  const GroupRole = IDL.Record({
    'permissions' : IDL.Vec(Permission),
    'name' : IDL.Text,
    'color' : IDL.Text,
    'protected' : IDL.Bool,
    'index' : IDL.Opt(IDL.Nat64),
  });
  const GroupResponse = IDL.Record({
    'updated_on' : IDL.Nat64,
    'banner_image' : Asset,
    'owner' : IDL.Principal,
    'name' : IDL.Text,
    'matrix_space_id' : IDL.Text,
    'tags' : IDL.Vec(IDL.Nat32),
    'description' : IDL.Text,
    'created_by' : IDL.Principal,
    'created_on' : IDL.Nat64,
    'website' : IDL.Text,
    'privacy' : Privacy,
    'image' : Asset,
    'identifier' : IDL.Principal,
    'member_count' : IDL.Nat64,
    'location' : Location,
    'roles' : IDL.Vec(GroupRole),
    'is_deleted' : IDL.Bool,
  });
  const PagedResponse = IDL.Record({
    'total' : IDL.Nat64,
    'data' : IDL.Vec(GroupResponse),
    'page' : IDL.Nat64,
    'limit' : IDL.Nat64,
    'number_of_pages' : IDL.Nat64,
  });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
  });
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  return IDL.Service({
    '__get_candid_interface_tmp_hack' : IDL.Func([], [IDL.Text], ['query']),
    'accept_cycles' : IDL.Func([], [IDL.Nat64], []),
    'close_child_canister_and_spawn_sibling' : IDL.Func(
        [IDL.Nat64, IDL.Vec(IDL.Nat8)],
        [Result],
        [],
      ),
    'get_available_canister' : IDL.Func([], [Result_1], ['query']),
    'get_canisters' : IDL.Func(
        [],
        [IDL.Vec(ScalableCanisterDetails)],
        ['query'],
      ),
    'get_groups' : IDL.Func(
        [IDL.Nat64, IDL.Nat64, IDL.Vec(GroupFilter), FilterType, GroupSort],
        [PagedResponse],
        ['query'],
      ),
    'get_latest_wasm_version' : IDL.Func([], [WasmVersion], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
