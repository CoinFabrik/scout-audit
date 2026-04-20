#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use std::collections::{HashMap, HashSet};

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    analysis::{is_soroban_function, FunctionCallVisitor},
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span,
};

const LINT_MESSAGE: &str =
    "Recursive call flows can recurse indefinitely and exhaust gas or revert";

#[expose_lint_info]
pub static INFINITE_RECURSION_OVER_STORAGE_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Recursive call flows can recurse indefinitely and exhaust gas or revert. The detector flags recursive cycles reachable from Soroban entrypoints, including direct self-recursion and mutual recursion.",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/soroban/infinite-recursion-over-storage",
    vulnerability_class: VulnerabilityClass::DoS,
};

dylint_linting::impl_late_lint! {
    pub INFINITE_RECURSION_OVER_STORAGE,
    Warn,
    LINT_MESSAGE,
    InfiniteRecursionOverStorage::default()
}

#[derive(Default)]
struct InfiniteRecursionOverStorage {
    checked_functions: HashSet<String>,
    functions: HashSet<DefId>,
    function_call_graph: HashMap<DefId, HashSet<DefId>>,
    call_edges_with_spans: Vec<CallEdge>,
}

#[derive(Clone, Copy)]
struct CallEdge {
    caller: DefId,
    callee: DefId,
    span: Span,
}

impl<'tcx> LateLintPass<'tcx> for InfiniteRecursionOverStorage {
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        let reachable = collect_reachable_entrypoints(
            cx,
            &self.checked_functions,
            &self.functions,
            &self.function_call_graph,
        );

        if reachable.is_empty() {
            return;
        }

        let reachable_graph = build_reachable_graph(&reachable, &self.function_call_graph);
        let recursive_sccs = compute_recursive_sccs(&reachable, &reachable_graph);

        for component in recursive_sccs {
            let component_members: HashSet<DefId> = component.into_iter().collect();
            for edge in &self.call_edges_with_spans {
                if component_members.contains(&edge.caller)
                    && component_members.contains(&edge.callee)
                {
                    span_lint_and_help(
                        cx,
                        INFINITE_RECURSION_OVER_STORAGE,
                        edge.span,
                        LINT_MESSAGE,
                        None,
                        "Break the recursive cycle so the entrypoint cannot recurse indefinitely.",
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
        self.functions.insert(def_id);

        if span.from_expansion() {
            return;
        }

        let mut function_call_visitor =
            FunctionCallVisitor::new(cx, def_id, &mut self.function_call_graph);
        function_call_visitor.visit_body(body);

        let mut call_edge_visitor = CallEdgeVisitor {
            cx,
            caller: def_id,
            edges: Vec::new(),
        };
        call_edge_visitor.visit_body(body);
        self.call_edges_with_spans.extend(call_edge_visitor.edges);
    }
}

fn collect_reachable_entrypoints(
    cx: &LateContext<'_>,
    checked_functions: &HashSet<String>,
    functions: &HashSet<DefId>,
    graph: &HashMap<DefId, HashSet<DefId>>,
) -> HashSet<DefId> {
    let mut reachable = HashSet::new();

    for function in functions {
        if !is_soroban_function(cx, checked_functions, function) {
            continue;
        }

        let mut stack = vec![*function];
        while let Some(current) = stack.pop() {
            if !reachable.insert(current) {
                continue;
            }

            if let Some(callees) = graph.get(&current) {
                for callee in callees {
                    if functions.contains(callee) && !reachable.contains(callee) {
                        stack.push(*callee);
                    }
                }
            }
        }
    }

    reachable
}

fn build_reachable_graph(
    reachable: &HashSet<DefId>,
    graph: &HashMap<DefId, HashSet<DefId>>,
) -> HashMap<DefId, HashSet<DefId>> {
    let mut reachable_graph = HashMap::new();

    for function in reachable {
        let children = graph
            .get(function)
            .into_iter()
            .flat_map(|callees| callees.iter().copied())
            .filter(|callee| reachable.contains(callee))
            .collect::<HashSet<_>>();
        reachable_graph.insert(*function, children);
    }

    reachable_graph
}

fn compute_recursive_sccs(
    reachable: &HashSet<DefId>,
    graph: &HashMap<DefId, HashSet<DefId>>,
) -> Vec<Vec<DefId>> {
    let mut tarjan = TarjanState::new(graph);

    for node in reachable {
        if !tarjan.indexes.contains_key(node) {
            tarjan.strong_connect(*node);
        }
    }

    tarjan
        .components
        .into_iter()
        .filter(|component| {
            if component.len() > 1 {
                return true;
            }

            component
                .first()
                .and_then(|node| graph.get(node).map(|children| children.contains(node)))
                .unwrap_or(false)
        })
        .collect()
}

struct TarjanState<'a> {
    graph: &'a HashMap<DefId, HashSet<DefId>>,
    index: usize,
    stack: Vec<DefId>,
    on_stack: HashSet<DefId>,
    indexes: HashMap<DefId, usize>,
    lowlinks: HashMap<DefId, usize>,
    components: Vec<Vec<DefId>>,
}

impl<'a> TarjanState<'a> {
    fn new(graph: &'a HashMap<DefId, HashSet<DefId>>) -> Self {
        Self {
            graph,
            index: 0,
            stack: Vec::new(),
            on_stack: HashSet::new(),
            indexes: HashMap::new(),
            lowlinks: HashMap::new(),
            components: Vec::new(),
        }
    }

    fn strong_connect(&mut self, node: DefId) {
        self.indexes.insert(node, self.index);
        self.lowlinks.insert(node, self.index);
        self.index += 1;
        self.stack.push(node);
        self.on_stack.insert(node);

        if let Some(children) = self.graph.get(&node) {
            for child in children {
                if !self.indexes.contains_key(child) {
                    self.strong_connect(*child);
                    let child_lowlink = *self.lowlinks.get(child).unwrap();
                    let node_lowlink = self.lowlinks.get_mut(&node).unwrap();
                    *node_lowlink = (*node_lowlink).min(child_lowlink);
                } else if self.on_stack.contains(child) {
                    let child_index = *self.indexes.get(child).unwrap();
                    let node_lowlink = self.lowlinks.get_mut(&node).unwrap();
                    *node_lowlink = (*node_lowlink).min(child_index);
                }
            }
        }

        if self.indexes.get(&node) == self.lowlinks.get(&node) {
            let mut component = Vec::new();
            while let Some(current) = self.stack.pop() {
                self.on_stack.remove(&current);
                component.push(current);
                if current == node {
                    break;
                }
            }
            self.components.push(component);
        }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for CallEdgeVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if let Some(def_id) = resolve_call_def_id(self.cx, expr) {
            self.edges.push(CallEdge {
                caller: self.caller,
                callee: def_id,
                span: expr.span,
            });
        }

        walk_expr(self, expr);
    }
}

fn resolve_call_def_id(cx: &LateContext<'_>, expr: &Expr<'_>) -> Option<DefId> {
    match expr.kind {
        ExprKind::Call(callee_expr, _) => match callee_expr.kind {
            ExprKind::Path(ref qpath) => resolve_qpath_def_id(cx, qpath, callee_expr.hir_id),
            _ => None,
        },
        ExprKind::MethodCall(..) => cx.typeck_results().type_dependent_def_id(expr.hir_id),
        _ => None,
    }
}

struct CallEdgeVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    caller: DefId,
    edges: Vec<CallEdge>,
}

fn resolve_qpath_def_id(
    cx: &LateContext<'_>,
    qpath: &QPath<'_>,
    hir_id: rustc_hir::HirId,
) -> Option<DefId> {
    cx.qpath_res(qpath, hir_id).opt_def_id()
}
