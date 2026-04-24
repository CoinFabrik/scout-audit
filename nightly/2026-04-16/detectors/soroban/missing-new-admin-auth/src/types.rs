use rustc_hir::HirId;
use rustc_span::{def_id::DefId, Span};

#[derive(Clone, Debug)]
pub struct ParamInfo {
    pub hir_id: HirId,
    pub is_address: bool,
    pub is_privileged_name: bool,
}

/// When a new admin/owner is set (storage.set with privileged key)
#[derive(Clone, Debug)]
pub struct Sink {
    pub param_index: Option<usize>,
    pub span: Span,
}

/// When `require_auth` is called on an address
#[derive(Clone, Debug)]
pub struct AuthEvent {
    pub param_index: Option<usize>,
    pub span: Span,
    pub is_current_admin: bool,
}

// TODO: we could treat MethodCalls as call sites in the future
/// A function call site tracking argument-to-parameter mapping
#[derive(Clone, Debug)]
pub struct CallSite {
    pub callee_def_id: DefId,
    /// Maps callee param index -> caller param index
    pub arg_to_param: Vec<Option<usize>>,
    pub span: Span,
}
