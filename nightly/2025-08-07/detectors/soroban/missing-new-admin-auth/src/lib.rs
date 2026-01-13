#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    analysis::{self, get_node_type_opt, is_soroban_address, is_soroban_function},
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use edit_distance::edit_distance;
use if_chain::if_chain;
use rustc_hir::{
    def::Res,
    intravisit::{walk_expr, walk_local, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, HirId, LetStmt, Param, PatKind, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span, Symbol,
};
use std::collections::{HashMap, HashSet};

const LINT_MESSAGE: &str = "New admin/owner address must sign before being stored";

#[expose_lint_info]
pub static MISSING_NEW_ADMIN_AUTH_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "When updating admin or owner, the incoming address should also sign to prevent accidental bricking due to a mistaken address.",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/soroban/missing-new-admin-auth",
    vulnerability_class: VulnerabilityClass::Authorization,
};

dylint_linting::impl_late_lint! {
    pub MISSING_NEW_ADMIN_AUTH,
    Warn,
    LINT_MESSAGE,
    MissingNewAdminAuth::default()
}

#[derive(Clone, Debug)]
struct ParamInfo {
    is_address: bool,
    is_privileged_name: bool,
}

#[derive(Clone, Debug)]
struct SinkRecord {
    span: Span,
    param_index: Option<usize>,
}

#[derive(Clone, Debug)]
struct CallInfo {
    callee: DefId,
    arg_param_map: Vec<Option<usize>>,
}

#[derive(Default)]
struct MissingNewAdminAuth {
    checked_functions: HashSet<String>,
    local_auth: HashMap<DefId, HashSet<usize>>,
    has_any_auth: HashMap<DefId, bool>,
    param_infos: HashMap<DefId, Vec<ParamInfo>>,
    sinks: HashMap<DefId, Vec<SinkRecord>>,
    call_mappings: HashMap<DefId, Vec<CallInfo>>,
}

impl<'tcx> LateLintPass<'tcx> for MissingNewAdminAuth {
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        // Only analyze call paths reachable from Soroban contract entrypoints.
        let entrypoints: HashSet<DefId> = self
            .param_infos
            .keys()
            .copied()
            .filter(|def_id| is_soroban_function(cx, &self.checked_functions, def_id))
            .collect();

        let reachable = collect_reachable(&self.call_mappings, &entrypoints);
        let mut effective_auth = self.local_auth.clone();
        let mut effective_has_auth = self.has_any_auth.clone();
        let mut changed = true;

        while changed {
            changed = false;
            for (caller, calls) in &self.call_mappings {
                let caller_auth = effective_auth.get(caller).cloned().unwrap_or_default();
                let caller_has_auth = effective_has_auth.get(caller).copied().unwrap_or(false);
                for call in calls {
                    // Propagate param-level auth
                    let callee_auth = effective_auth.entry(call.callee).or_default();
                    for (arg_index, caller_param) in call.arg_param_map.iter().enumerate() {
                        if let Some(caller_param) = caller_param {
                            if caller_auth.contains(caller_param) && callee_auth.insert(arg_index) {
                                changed = true;
                            }
                        }
                    }
                    // Propagate "has any auth" context: if caller has auth, callee is in update context
                    if caller_has_auth {
                        let callee_has_auth =
                            effective_has_auth.entry(call.callee).or_insert(false);
                        if !*callee_has_auth {
                            *callee_has_auth = true;
                            changed = true;
                        }
                    }
                }
            }
        }

        for (def_id, sinks) in &self.sinks {
            if !reachable.contains(def_id) {
                continue;
            }
            // Skip if not in update context (no auth required anywhere in the call chain)
            let is_update_context = effective_has_auth.get(def_id).copied().unwrap_or(false);
            if !is_update_context {
                continue;
            }
            let authed_params = effective_auth.get(def_id);
            let param_infos = self.param_infos.get(def_id);
            for sink in sinks {
                let Some(param_index) = sink.param_index else {
                    // No direct trace - skip
                    continue;
                };
                // Check param is Address AND has privileged name
                let param_valid = param_infos
                    .and_then(|infos| infos.get(param_index))
                    .map(|info| info.is_address && info.is_privileged_name)
                    .unwrap_or(false);
                if !param_valid {
                    continue;
                }
                let is_authed = authed_params
                    .map(|set| set.contains(&param_index))
                    .unwrap_or(false);
                if !is_authed {
                    span_lint_and_help(
                        cx,
                        MISSING_NEW_ADMIN_AUTH,
                        sink.span,
                        LINT_MESSAGE,
                        None,
                        "Require the incoming admin/owner address to sign before storing it",
                    );
                }
            }
        }
    }

    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        span: Span,
        local_def_id: LocalDefId,
    ) {
        let def_id = local_def_id.to_def_id();
        self.checked_functions.insert(cx.tcx.def_path_str(def_id));

        if span.from_expansion() {
            return;
        }

        let (param_infos, param_hir_ids) = collect_param_infos(cx, body.params);

        let mut visitor = MissingNewAdminAuthVisitor::new(cx, param_hir_ids);
        visitor.visit_body(body);

        self.local_auth.insert(def_id, visitor.local_auth);
        self.has_any_auth.insert(def_id, visitor.has_any_auth);
        self.param_infos.insert(def_id, param_infos);
        self.sinks.insert(def_id, visitor.sinks);
        self.call_mappings.insert(def_id, visitor.call_mappings);
    }
}

struct MissingNewAdminAuthVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    param_hir_ids: HashMap<HirId, usize>,
    // Track simple `let x = param` aliases to keep param tracing stable.
    local_aliases: HashMap<HirId, usize>,
    local_auth: HashSet<usize>,
    has_any_auth: bool,
    sinks: Vec<SinkRecord>,
    // Map callee args back to caller params for interprocedural propagation.
    call_mappings: Vec<CallInfo>,
}

impl<'a, 'tcx> MissingNewAdminAuthVisitor<'a, 'tcx> {
    fn new(cx: &'a LateContext<'tcx>, param_hir_ids: HashMap<HirId, usize>) -> Self {
        Self {
            cx,
            param_hir_ids,
            local_aliases: HashMap::new(),
            local_auth: HashSet::new(),
            has_any_auth: false,
            sinks: Vec::new(),
            call_mappings: Vec::new(),
        }
    }

    // Resolve an expression back to a parameter index when possible.
    fn resolve_param_index(&self, expr: &Expr<'_>) -> Option<usize> {
        match expr.kind {
            ExprKind::AddrOf(_, _, inner) => self.resolve_param_index(inner),
            ExprKind::Path(QPath::Resolved(_, path)) => match path.res {
                Res::Local(hir_id) => self
                    .param_hir_ids
                    .get(&hir_id)
                    .copied()
                    .or_else(|| self.local_aliases.get(&hir_id).copied()),
                _ => None,
            },
            ExprKind::MethodCall(path_segment, receiver, args, _) => {
                if path_segment.ident.name == Symbol::intern("clone") && args.is_empty() {
                    self.resolve_param_index(receiver)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn record_call(&mut self, callee: DefId, args: impl Iterator<Item = &'tcx Expr<'tcx>>) {
        if !callee.is_local() {
            return;
        }
        let arg_param_map = args.map(|arg| self.resolve_param_index(arg)).collect();
        self.call_mappings.push(CallInfo {
            callee,
            arg_param_map,
        });
    }
}

impl<'tcx> Visitor<'tcx> for MissingNewAdminAuthVisitor<'_, 'tcx> {
    fn visit_local(&mut self, local: &'tcx LetStmt<'tcx>) {
        if let PatKind::Binding(_, _, _ident, _) = local.pat.kind {
            if let Some(init) = &local.init {
                if let Some(param_index) = self.resolve_param_index(init) {
                    self.local_aliases.insert(local.pat.hir_id, param_index);
                }
            }
        }
        walk_local(self, local);
    }

    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(path_segment, receiver, args, _) = &expr.kind {
            let name = path_segment.ident.name;
            if name == Symbol::intern("require_auth")
                || name == Symbol::intern("require_auth_for_args")
            {
                self.has_any_auth = true;
                if let Some(param_index) = self.resolve_param_index(receiver) {
                    self.local_auth.insert(param_index);
                }
            }

            let receiver_type = get_node_type_opt(self.cx, &receiver.hir_id);
            if name == Symbol::intern("set")
                && receiver_type.is_some_and(|ty| {
                    analysis::is_soroban_storage(self.cx, ty, analysis::SorobanStorageType::Any)
                })
                && args.len() >= 2
                && is_privileged_key(&args[0])
            {
                let param_index = self.resolve_param_index(&args[1]);
                self.sinks.push(SinkRecord {
                    span: expr.span,
                    param_index,
                });
            }

            if let Some(def_id) = self.cx.typeck_results().type_dependent_def_id(expr.hir_id) {
                let iter = std::iter::once(*receiver).chain(args.iter());
                self.record_call(def_id, iter);
            }
        }

        if let ExprKind::Call(call_expr, args) = &expr.kind {
            if let ExprKind::Path(ref qpath) = call_expr.kind {
                if let Some(def_id) = self.cx.qpath_res(qpath, call_expr.hir_id).opt_def_id() {
                    self.record_call(def_id, args.iter());
                }
            }
        }

        walk_expr(self, expr);
    }
}

fn collect_param_infos<'tcx>(
    cx: &LateContext<'tcx>,
    params: &'tcx [Param<'tcx>],
) -> (Vec<ParamInfo>, HashMap<HirId, usize>) {
    let mut param_infos = Vec::new();
    let mut param_hir_ids = HashMap::new();

    for (index, param) in params.iter().enumerate() {
        let (name, binding_hir_id) = match param.pat.kind {
            PatKind::Binding(_, _, ident, _) => (ident.name.as_str().to_string(), param.pat.hir_id),
            _ => (String::new(), param.pat.hir_id),
        };

        let is_address =
            get_node_type_opt(cx, &param.hir_id).is_some_and(|ty| is_soroban_address(cx, ty));
        let is_privileged_name = is_privileged_name(&name);

        param_hir_ids.insert(binding_hir_id, index);
        param_infos.push(ParamInfo {
            is_address,
            is_privileged_name,
        });
    }

    (param_infos, param_hir_ids)
}

fn is_privileged_key(expr: &Expr<'_>) -> bool {
    match expr.kind {
        ExprKind::AddrOf(_, _, inner) => is_privileged_key(inner),
        ExprKind::Path(QPath::Resolved(_, path)) => {
            if_chain! {
                if let Res::Def(_, _) = path.res;
                if let Some(seg) = path.segments.last();
                let name = seg.ident.name.as_str();
                then {
                    return is_privileged_name(name);
                }
            }
            false
        }
        _ => false,
    }
}

fn is_privileged_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let targets = [
        "admin",
        "owner",
        "new_admin",
        "new_owner",
        "newadmin",
        "newowner",
        "next_admin",
        "next_owner",
        "nextadmin",
        "nextowner",
        "pending_admin",
        "pending_owner",
        "pendingadmin",
        "pendingowner",
    ];
    targets
        .iter()
        .any(|target| edit_distance(&lower, target) <= 1)
}

fn collect_reachable(
    call_mappings: &HashMap<DefId, Vec<CallInfo>>,
    entrypoints: &HashSet<DefId>,
) -> HashSet<DefId> {
    let mut reachable = HashSet::new();
    let mut stack: Vec<DefId> = entrypoints.iter().copied().collect();

    while let Some(def_id) = stack.pop() {
        if !reachable.insert(def_id) {
            continue;
        }
        if let Some(calls) = call_mappings.get(&def_id) {
            for call in calls {
                stack.push(call.callee);
            }
        }
    }

    reachable
}
