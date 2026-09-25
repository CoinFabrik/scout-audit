#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::{diagnostics::span_lint_and_help, peel_blocks, sym};
use common::{
    analysis::{
        get_expr_hir_id_opt, get_node_type_opt, is_soroban_address, is_soroban_function,
        verify_token_interface_function, FunctionCallVisitor,
    },
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::{
    intravisit::{walk_expr, walk_local, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, HirId, LetStmt, PatKind, UnOp,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span,
};
use std::collections::{HashMap, HashSet};

const LINT_MESSAGE: &str =
    "Delegated token operations should not require authorization from both spender and from";
const SPENDER_PARAM_INDEX: usize = 1;
const FROM_PARAM_INDEX: usize = 2;

#[expose_lint_info]
pub static DELEGATED_SPENDING_FROM_AUTH_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Delegated token operations such as transfer_from and burn_from should be authorized by the spender only. Requiring authorization from the from account defeats the allowance mechanism and can make delegated transfers or burns unusable.",
    severity: Severity::Minor,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/soroban/delegated-spending-from-auth",
    vulnerability_class: VulnerabilityClass::Authorization,
};

dylint_linting::impl_late_lint! {
    pub DELEGATED_SPENDING_FROM_AUTH,
    Warn,
    LINT_MESSAGE,
    DelegatedSpendingFromAuth::default()
}

#[derive(Clone, Copy)]
struct DelegatedCandidate {
    spender_param_index: usize,
    from_param_index: usize,
}

#[derive(Clone)]
struct ParamInfo {
    hir_id: HirId,
}

#[derive(Clone)]
struct AuthEvent {
    param_index: usize,
    span: Span,
}

#[derive(Clone)]
struct CallSite {
    callee_def_id: DefId,
    /// `arg_to_param[callee_param_index] = Some(caller_param_index)` when the callee parameter
    /// receives a value that can be traced back to a caller parameter.
    arg_to_param: Vec<Option<usize>>,
    span: Span,
}

#[derive(Default)]
struct DelegatedSpendingFromAuth {
    checked_functions: HashSet<String>,
    candidates: HashMap<DefId, DelegatedCandidate>,
    params: HashMap<DefId, Vec<ParamInfo>>,
    direct_auth_events: HashMap<DefId, Vec<AuthEvent>>,
    call_sites: HashMap<DefId, Vec<CallSite>>,
    function_call_graph: HashMap<DefId, HashSet<DefId>>,
}

impl<'tcx> LateLintPass<'tcx> for DelegatedSpendingFromAuth {
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        let checked_names = self.checked_functions.clone();

        for (def_id, candidate) in &self.candidates {
            if !is_soroban_function(cx, &checked_names, def_id) {
                continue;
            }

            let reachable = collect_reachable_functions(*def_id, &self.function_call_graph);
            let summaries = compute_auth_summaries(
                &reachable,
                &self.params,
                &self.direct_auth_events,
                &self.call_sites,
            );
            let from_direct_auth_span =
                direct_auth_span(&self.direct_auth_events, def_id, candidate.from_param_index);

            // Delegated entrypoints must require auth from the spender somewhere in the
            // reachable flow, even if that check is factored into a local helper.
            if !summary_requires_auth(&summaries, def_id, candidate.spender_param_index) {
                continue;
            }

            // The finding is present only if the reachable delegated flow also requires auth
            // from `from`, either directly in the entrypoint or after propagating through local
            // helpers.
            if !summary_requires_auth(&summaries, def_id, candidate.from_param_index) {
                continue;
            }

            let span = from_direct_auth_span
                .or_else(|| {
                    find_indirect_auth_span(
                        def_id,
                        candidate.from_param_index,
                        &summaries,
                        &self.call_sites,
                    )
                })
                .or_else(|| cx.tcx.hir_span_if_local(*def_id));

            if let Some(span) = span {
                span_lint_and_help(
                    cx,
                    DELEGATED_SPENDING_FROM_AUTH,
                    span,
                    LINT_MESSAGE,
                    None,
                    "keep spender authorization in delegated flows, but update allowance through an internal helper that does not call `from.require_auth()`",
                );
            }
        }
    }

    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        fn_decl: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        span: Span,
        local_def_id: LocalDefId,
    ) {
        let def_id = local_def_id.to_def_id();
        self.checked_functions.insert(cx.tcx.def_path_str(def_id));

        if span.from_expansion() {
            return;
        }

        let mut function_call_visitor =
            FunctionCallVisitor::new(cx, def_id, &mut self.function_call_graph);
        function_call_visitor.visit_body(body);

        let params = collect_param_info(body);
        let mut visitor = DelegatedSpendingFromAuthVisitor::new(cx, params.clone());
        visitor.visit_body(body);

        self.params.insert(def_id, params);
        self.direct_auth_events.insert(def_id, visitor.auth_events);
        self.call_sites.insert(def_id, visitor.call_sites);

        if let Some(candidate) = candidate_for_fn(cx, def_id, fn_decl, body) {
            self.candidates.insert(def_id, candidate);
        }
    }
}

fn candidate_for_fn<'tcx>(
    cx: &LateContext<'tcx>,
    def_id: DefId,
    fn_decl: &'tcx FnDecl<'tcx>,
    body: &'tcx Body<'tcx>,
) -> Option<DelegatedCandidate> {
    let fn_name = cx.tcx.def_path_str(def_id);
    if !verify_token_interface_function(fn_name.clone(), fn_decl.inputs, fn_decl.output) {
        return None;
    }

    let last_segment = fn_name.split("::").last()?;
    if !matches!(last_segment, "transfer_from" | "burn_from") {
        return None;
    }

    let spender_is_address = body
        .params
        .get(SPENDER_PARAM_INDEX)
        .and_then(|param| get_node_type_opt(cx, &param.pat.hir_id))
        .map(|ty| is_soroban_address(cx, ty))
        .unwrap_or(false);
    let from_is_address = body
        .params
        .get(FROM_PARAM_INDEX)
        .and_then(|param| get_node_type_opt(cx, &param.pat.hir_id))
        .map(|ty| is_soroban_address(cx, ty))
        .unwrap_or(false);

    if !(spender_is_address && from_is_address) {
        return None;
    }

    Some(DelegatedCandidate {
        spender_param_index: SPENDER_PARAM_INDEX,
        from_param_index: FROM_PARAM_INDEX,
    })
}

fn collect_param_info(body: &Body<'_>) -> Vec<ParamInfo> {
    body.params
        .iter()
        .map(|param| ParamInfo {
            hir_id: param.pat.hir_id,
        })
        .collect()
}

fn collect_reachable_functions(
    entrypoint: DefId,
    function_call_graph: &HashMap<DefId, HashSet<DefId>>,
) -> HashSet<DefId> {
    let mut reachable = HashSet::new();
    let mut stack = vec![entrypoint];

    while let Some(current) = stack.pop() {
        if !reachable.insert(current) {
            continue;
        }

        if let Some(callees) = function_call_graph.get(&current) {
            for callee in callees {
                if !reachable.contains(callee) {
                    stack.push(*callee);
                }
            }
        }
    }

    reachable
}

fn compute_auth_summaries(
    reachable: &HashSet<DefId>,
    params: &HashMap<DefId, Vec<ParamInfo>>,
    direct_auth_events: &HashMap<DefId, Vec<AuthEvent>>,
    call_sites: &HashMap<DefId, Vec<CallSite>>,
) -> HashMap<DefId, Vec<bool>> {
    // For each reachable function, summary[param_index] answers:
    // "does this function require auth on this parameter directly or through local helpers?"
    let mut summaries: HashMap<DefId, Vec<bool>> = reachable
        .iter()
        .filter_map(|def_id| {
            params
                .get(def_id)
                .map(|fn_params| (*def_id, vec![false; fn_params.len()]))
        })
        .collect();

    loop {
        let mut changed = false;

        for def_id in reachable {
            let Some(current_summary) = summaries.get(def_id).cloned() else {
                continue;
            };

            let mut next_summary = current_summary;

            if let Some(events) = direct_auth_events.get(def_id) {
                for event in events {
                    if let Some(slot) = next_summary.get_mut(event.param_index) {
                        *slot = true;
                    }
                }
            }

            if let Some(calls) = call_sites.get(def_id) {
                for call_site in calls {
                    let Some(callee_summary) = summaries.get(&call_site.callee_def_id) else {
                        continue;
                    };

                    for (callee_param_index, caller_param_index) in
                        call_site.arg_to_param.iter().enumerate()
                    {
                        let Some(caller_param_index) = caller_param_index else {
                            continue;
                        };

                        if callee_summary
                            .get(callee_param_index)
                            .copied()
                            .unwrap_or(false)
                        {
                            if let Some(slot) = next_summary.get_mut(*caller_param_index) {
                                *slot = true;
                            }
                        }
                    }
                }
            }

            if summaries.get(def_id) != Some(&next_summary) {
                summaries.insert(*def_id, next_summary);
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    summaries
}

fn summary_requires_auth(
    summaries: &HashMap<DefId, Vec<bool>>,
    def_id: &DefId,
    param_index: usize,
) -> bool {
    summaries
        .get(def_id)
        .and_then(|summary| summary.get(param_index))
        .copied()
        .unwrap_or(false)
}

fn direct_auth_span(
    direct_auth_events: &HashMap<DefId, Vec<AuthEvent>>,
    def_id: &DefId,
    param_index: usize,
) -> Option<Span> {
    // Returns the first direct `require_auth`/`require_auth_for_args` span for this parameter.
    direct_auth_events.get(def_id).and_then(|events| {
        events
            .iter()
            .find(|event| event.param_index == param_index)
            .map(|event| event.span)
    })
}

fn find_indirect_auth_span(
    def_id: &DefId,
    caller_param_index: usize,
    summaries: &HashMap<DefId, Vec<bool>>,
    call_sites: &HashMap<DefId, Vec<CallSite>>,
) -> Option<Span> {
    // Finds the helper callsite where the caller's parameter first flows into a callee
    // parameter whose reachability summary says that auth is required.
    call_sites.get(def_id).and_then(|sites| {
        sites.iter().find_map(|site| {
            let flows_to_auth =
                site.arg_to_param
                    .iter()
                    .enumerate()
                    .any(|(callee_param_index, mapped_param)| {
                        mapped_param == &Some(caller_param_index)
                            && summaries
                                .get(&site.callee_def_id)
                                .and_then(|summary| summary.get(callee_param_index))
                                .copied()
                                .unwrap_or(false)
                    });

            flows_to_auth.then_some(site.span)
        })
    })
}

struct DelegatedSpendingFromAuthVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    param_by_hir: HashMap<HirId, usize>,
    aliases: HashMap<HirId, HirId>,
    auth_events: Vec<AuthEvent>,
    call_sites: Vec<CallSite>,
}

impl<'a, 'tcx> DelegatedSpendingFromAuthVisitor<'a, 'tcx> {
    fn new(cx: &'a LateContext<'tcx>, params: Vec<ParamInfo>) -> Self {
        let param_by_hir = params
            .iter()
            .enumerate()
            .map(|(index, param)| (param.hir_id, index))
            .collect();

        Self {
            cx,
            param_by_hir,
            aliases: HashMap::new(),
            auth_events: vec![],
            call_sites: vec![],
        }
    }
}

impl<'tcx> Visitor<'tcx> for DelegatedSpendingFromAuthVisitor<'_, 'tcx> {
    fn visit_local(&mut self, local: &'tcx LetStmt<'tcx>) {
        if let PatKind::Binding(_, _, _, _) = local.pat.kind {
            if let Some(init) = local.init {
                if let Some(param_hir_id) =
                    resolve_expr_to_param(init, &self.aliases, &self.param_by_hir)
                {
                    self.aliases.insert(local.pat.hir_id, param_hir_id);
                }
            }
        }

        walk_local(self, local);
    }

    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if let ExprKind::Call(callee, args) = expr.kind {
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

        if let ExprKind::MethodCall(path_segment, receiver, _, _) = expr.kind {
            let method_name = path_segment.ident.name;
            let is_auth_method = method_name.as_str() == "require_auth"
                || method_name.as_str() == "require_auth_for_args";

            if is_auth_method {
                if let Some(param_hir_id) =
                    resolve_expr_to_param(receiver, &self.aliases, &self.param_by_hir)
                {
                    if let Some(param_index) = self.param_by_hir.get(&param_hir_id).copied() {
                        self.auth_events.push(AuthEvent {
                            param_index,
                            span: expr.span,
                        });
                    }
                }
            }
        }

        walk_expr(self, expr);
    }
}

fn resolve_expr_to_param(
    expr: &Expr<'_>,
    aliases: &HashMap<HirId, HirId>,
    param_by_hir: &HashMap<HirId, usize>,
) -> Option<HirId> {
    let hir_id = get_expr_hir_id_stripped(expr)?;
    let resolved = resolve_alias(hir_id, aliases);
    param_by_hir.contains_key(&resolved).then_some(resolved)
}

fn get_expr_hir_id_stripped(expr: &Expr<'_>) -> Option<HirId> {
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

fn strip_identity<'tcx>(expr: &'tcx Expr<'tcx>) -> &'tcx Expr<'tcx> {
    let expr = peel_blocks(expr);

    match expr.kind {
        ExprKind::AddrOf(_, _, inner) => strip_identity(inner),
        ExprKind::Unary(UnOp::Deref, inner) => strip_identity(inner),
        ExprKind::MethodCall(path_segment, receiver, _, _)
            if matches!(
                path_segment.ident.name,
                sym::clone | sym::to_owned | sym::into
            ) =>
        {
            strip_identity(receiver)
        }
        _ => expr,
    }
}
