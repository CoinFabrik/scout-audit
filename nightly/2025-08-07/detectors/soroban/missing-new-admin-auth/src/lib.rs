#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_middle;
extern crate rustc_span;

mod mir_auth_flow;
mod types;
mod utils;

use clippy_utils::sym;
use common::{
    analysis::{
        self, get_expr_hir_id_opt, get_node_type_opt, is_soroban_address, is_soroban_function,
        FunctionCallVisitor,
    },
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_hir::{
    def::Res,
    intravisit::{walk_expr, walk_local, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, HirId, LetStmt, PatKind, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span, Symbol,
};
use std::collections::{HashMap, HashSet};

use crate::{
    mir_auth_flow::{compute_summary_for_fn, lint_sinks_for_fn, FnSummary},
    types::{AuthEvent, CallSite, ParamInfo, Sink},
    utils::{
        get_vec_slice, is_get_method, is_initialize_fn, is_privileged_name, is_unwrap_method,
        strip_identity,
    },
};

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

#[derive(Default)]
struct MissingNewAdminAuth {
    checked_functions: HashMap<String, DefId>,
    // Map functions -> parameters
    params: HashMap<DefId, Vec<ParamInfo>>,
    sinks: HashMap<DefId, Vec<Sink>>,
    auth_events: HashMap<DefId, Vec<AuthEvent>>,
    function_call_graph: HashMap<DefId, HashSet<DefId>>,
    call_sites: HashMap<DefId, Vec<CallSite>>,
}

impl<'tcx> LateLintPass<'tcx> for MissingNewAdminAuth {
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        let checked_names: HashSet<String> = self.checked_functions.keys().cloned().collect();
        self.checked_functions.iter().for_each(|(_, def_id)| {
            if !is_soroban_function(cx, &checked_names, def_id) {
                return;
            }

            // Collect reachable functions
            // Build reachable set for this entrypoint (DFS/BFS)
            let mut reachable: HashSet<DefId> = HashSet::new();
            let mut stack: Vec<DefId> = vec![*def_id];
            while let Some(current) = stack.pop() {
                if !reachable.insert(current) {
                    continue;
                }
                if let Some(callees) = self.function_call_graph.get(&current) {
                    for callee in callees {
                        if !reachable.contains(callee) {
                            stack.push(*callee);
                        }
                    }
                }
            }

            // On reachable, we have the functions that SHOULD be analyzed.
            // We can get sinks and authevents on each function, and we know its relationships
            // through the function_call_graph.

            // Now, we need to understand the safety of those sinks, composing everything we have.
            // Using the MIR with DataAnalysis

            let mut summaries: HashMap<DefId, FnSummary> = HashMap::new();
            for def_id in &reachable {
                if let Some(params) = self.params.get(def_id) {
                    summaries.insert(*def_id, FnSummary::empty(params.len()));
                }
            }

            // Fixpoint over summaries
            loop {
                let mut changed = false;

                for def_id in &reachable {
                    let Some(params) = self.params.get(def_id) else {
                        continue;
                    };
                    let Some(local_def_id) = def_id.as_local() else {
                        continue;
                    };

                    let body = cx.tcx.optimized_mir(local_def_id);
                    let events = get_vec_slice(&self.auth_events, def_id);
                    let callsites = get_vec_slice(&self.call_sites, def_id);

                    let new_summary =
                        compute_summary_for_fn(cx, body, params, events, callsites, &summaries);

                    if summaries.get(def_id) != Some(&new_summary) {
                        summaries.insert(*def_id, new_summary);
                        changed = true;
                    }
                }

                if !changed {
                    break;
                }
            }

            // Final lint pass
            for def_id in &reachable {
                let Some(params) = self.params.get(def_id) else {
                    continue;
                };
                let Some(local_def_id) = def_id.as_local() else {
                    continue;
                };

                let body = cx.tcx.optimized_mir(local_def_id);
                let events = get_vec_slice(&self.auth_events, def_id);
                let sinks = get_vec_slice(&self.sinks, def_id);
                let callsites = get_vec_slice(&self.call_sites, def_id);

                lint_sinks_for_fn(cx, body, params, events, sinks, callsites, &summaries);
            }
        });
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

        if is_initialize_fn(cx, def_id) {
            return;
        }
        self.checked_functions
            .insert(cx.tcx.def_path_str(def_id), def_id);

        if span.from_expansion() {
            return;
        }

        let mut function_call_visitor =
            FunctionCallVisitor::new(cx, def_id, &mut self.function_call_graph);
        function_call_visitor.visit_body(body);

        // Store params for the current function
        let params_info = collect_param_info(cx, body);
        let mut visitor = MissingNewAdminAuthVisitor::new(cx, params_info.clone());
        visitor.visit_body(body);

        self.call_sites.insert(def_id, visitor.call_sites);

        self.params.insert(def_id, params_info);
        self.sinks.insert(def_id, visitor.sinks);
        self.auth_events.insert(def_id, visitor.auth_events.clone());
    }
}

fn collect_param_info<'tcx>(cx: &LateContext<'tcx>, body: &'tcx Body<'tcx>) -> Vec<ParamInfo> {
    body.params
        .iter()
        .map(|param| {
            let name = match param.pat.kind {
                PatKind::Binding(_, _, ident, _) => ident.name,
                _ => Symbol::intern(""),
            };

            let is_address = get_node_type_opt(cx, &param.hir_id)
                .map(|ty| is_soroban_address(cx, ty))
                .unwrap_or(false);

            ParamInfo {
                hir_id: param.pat.hir_id,
                is_address,
                is_privileged_name: is_privileged_name(name.as_str()),
            }
        })
        .collect()
}

struct MissingNewAdminAuthVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    // Params within the current function
    params: Vec<ParamInfo>,
    // Map hir_id -> param index (for quick lookup)
    param_by_hir: HashMap<HirId, usize>,
    // Sinks: when a new admin/owner is set
    sinks: Vec<Sink>,
    // Auth events: when a param is required to be authenticated
    auth_events: Vec<AuthEvent>,
    // Aliases: when a param is assigned to another param
    aliases: HashMap<HirId, HirId>,
    // Current admin locals: params that hold the current admin (from storage.get)
    current_admin_locals: HashSet<HirId>,
    call_sites: Vec<CallSite>,
}

impl<'a, 'tcx> MissingNewAdminAuthVisitor<'a, 'tcx> {
    fn new(cx: &'a LateContext<'tcx>, params: Vec<ParamInfo>) -> Self {
        let param_by_hir = params
            .iter()
            .enumerate()
            .map(|(i, param)| (param.hir_id, i))
            .collect();
        Self {
            cx,
            params,
            param_by_hir,
            sinks: vec![],
            auth_events: vec![],
            aliases: HashMap::new(),
            current_admin_locals: HashSet::new(),
            call_sites: vec![],
        }
    }
}

// Resolve an expr to a param HirId (after aliasing + filtering).
fn resolve_expr_to_param<'tcx>(
    expr: &'tcx Expr<'tcx>,
    aliases: &HashMap<HirId, HirId>,
    param_by_hir: &HashMap<HirId, usize>,
) -> Option<HirId> {
    let local_id = get_expr_hir_id_stripped(expr)?;
    let resolved = resolve_alias(local_id, aliases);
    param_by_hir.contains_key(&resolved).then_some(resolved)
}

fn get_expr_hir_id_stripped<'tcx>(expr: &'tcx Expr<'tcx>) -> Option<HirId> {
    get_expr_hir_id_opt(strip_identity(expr))
}

fn resolve_alias(hir_id: HirId, aliases: &HashMap<HirId, HirId>) -> HirId {
    let mut current = hir_id;
    while let Some(next) = aliases.get(&current).copied() {
        if next == current {
            break;
        }
        current = next;
    }
    current
}

// Detect `storage.get(...).unwrap()` (or `get` alone) with privileged key
fn is_privileged_storage_get<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx Expr<'tcx>,
) -> Option<(&'tcx Expr<'tcx>, &'tcx Expr<'tcx>)> {
    let mut expr = strip_identity(expr);

    loop {
        match &expr.kind {
            ExprKind::MethodCall(seg, receiver, args, _) => {
                let name = seg.ident.name;

                if is_unwrap_method(name) || matches!(name, sym::clone | sym::to_owned | sym::into)
                {
                    expr = receiver;
                    continue;
                }

                if is_get_method(name) {
                    let key_expr = args.first()?;
                    let is_storage = get_node_type_opt(cx, &receiver.hir_id).is_some_and(|ty| {
                        analysis::is_soroban_storage(cx, ty, analysis::SorobanStorageType::Any)
                    });

                    return (is_storage && is_privileged_key(key_expr))
                        .then_some((receiver, key_expr));
                }

                return None;
            }
            _ => return None,
        }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for MissingNewAdminAuthVisitor<'a, 'tcx> {
    fn visit_local(&mut self, local: &'tcx LetStmt<'tcx>) {
        if let PatKind::Binding(_, _, _ident, _) = local.pat.kind {
            if let Some(init) = &local.init {
                // Save current admin locals, if the init is a storage.get with privileged key
                if is_privileged_storage_get(self.cx, init).is_some() {
                    self.current_admin_locals.insert(local.pat.hir_id);
                }

                // If the init is a local, add it to aliases
                if let Some(local_id) = get_expr_hir_id_stripped(init) {
                    let resolved = resolve_alias(local_id, &self.aliases);
                    if self.param_by_hir.contains_key(&resolved)
                        || self.current_admin_locals.contains(&resolved)
                    {
                        self.aliases.insert(local.pat.hir_id, resolved);
                    }
                }
            }
        }
        walk_local(self, local);
    }

    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if let ExprKind::Call(callee, args) = expr.kind {
            if let ExprKind::Closure(closure) = callee.kind {
                let body = self.cx.tcx.hir_body(closure.body);
                self.visit_body(body);
            }

            if let ExprKind::Path(ref qpath) = callee.kind {
                if let Some(callee_def_id) = self.cx.qpath_res(qpath, callee.hir_id).opt_def_id() {
                    let arg_to_param = args
                        .iter()
                        .map(|arg| {
                            resolve_expr_to_param(arg, &self.aliases, &self.param_by_hir)
                                .and_then(|hir_id| self.param_by_hir.get(&hir_id).copied())
                        })
                        .collect();

                    self.call_sites.push(CallSite {
                        callee_def_id,
                        arg_to_param,
                        span: expr.span,
                    });
                }
            }
        }

        if let ExprKind::MethodCall(path_segment, receiver, call_args, _) = expr.kind {
            let method_name = path_segment.ident.name;
            let receiver_ty = get_node_type_opt(self.cx, &receiver.hir_id);

            // Find all Sinks (when a new admin/owner is set)
            // We care about storage.set operations, where the key is privileged.
            // We should only record Sinks that come from a PARAMETER.
            if method_name == Symbol::intern("set")
                && receiver_ty.is_some_and(|ty| {
                    analysis::is_soroban_storage(self.cx, ty, analysis::SorobanStorageType::Any)
                })
                && call_args.len() >= 2
                && is_privileged_key(&call_args[0])
            {
                if let Some(param_hir_id) =
                    resolve_expr_to_param(&call_args[1], &self.aliases, &self.param_by_hir)
                {
                    if let Some(param_index) = self.param_by_hir.get(&param_hir_id).copied() {
                        if self
                            .params
                            .get(param_index)
                            .is_some_and(|p| p.is_address && p.is_privileged_name)
                        {
                            self.sinks.push(Sink {
                                param_index: Some(param_index),
                                span: expr.span,
                            });
                        }
                    }
                }
            }

            // Find all Auth Events (when a param is required to be authenticated)
            // We care about require_auth and require_auth_for_args operations, where the receiver is an address.
            // We should record Auth Events that come from both a parameter and locals.
            if (method_name == Symbol::intern("require_auth")
                || method_name == Symbol::intern("require_auth_for_args"))
                && receiver_ty.is_some_and(|ty| analysis::is_soroban_address(self.cx, ty))
            {
                if let Some(local_id) = get_expr_hir_id_stripped(receiver) {
                    let resolved = resolve_alias(local_id, &self.aliases);
                    let param_index = self.param_by_hir.get(&resolved).copied();
                    let is_current_admin = self.current_admin_locals.contains(&resolved);

                    if param_index.is_some() || is_current_admin {
                        self.auth_events.push(AuthEvent {
                            param_index,
                            span: expr.span,
                            is_current_admin,
                        });
                    }
                }
            }
        }
        walk_expr(self, expr);
    }
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
