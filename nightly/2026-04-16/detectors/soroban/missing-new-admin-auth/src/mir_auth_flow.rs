extern crate rustc_index;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_mir_dataflow;

use clippy_utils::diagnostics::span_lint_and_help;
use rustc_hir::def_id::DefId;
use rustc_index::{bit_set::DenseBitSet, newtype_index, Idx};
use rustc_lint::LateContext;
use rustc_middle::{
    mir::{self, BasicBlock, Body, Const, Location, Operand, TerminatorKind},
    ty::{self, TyCtxt, TyKind, TypingEnv},
};
use rustc_mir_dataflow::{fmt::DebugWithContext, Analysis, JoinSemiLattice};
use rustc_span::Span;
use std::collections::HashMap;

use crate::types::{AuthEvent, CallSite, ParamInfo, Sink};
use crate::{LINT_MESSAGE, MISSING_NEW_ADMIN_AUTH};

newtype_index! {
    #[orderable]
    #[debug_format = "p{}"]
    pub struct ParamIndex {}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthDomain {
    unauth_params: DenseBitSet<ParamIndex>,
    current_admin_unauth: bool,
}

impl JoinSemiLattice for AuthDomain {
    fn join(&mut self, other: &Self) -> bool {
        let mut changed = self.unauth_params.union(&other.unauth_params);
        if !self.current_admin_unauth && other.current_admin_unauth {
            self.current_admin_unauth = true;
            changed = true;
        }
        changed
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FnSummary {
    must_auth_params: DenseBitSet<ParamIndex>,
    must_auth_current_admin: bool,
}

impl FnSummary {
    pub fn empty(param_count: usize) -> Self {
        Self {
            must_auth_params: DenseBitSet::new_empty(param_count),
            must_auth_current_admin: false,
        }
    }
}

fn spans_overlap(a: Span, b: Span) -> bool {
    a.lo() <= b.hi() && b.lo() <= a.hi()
}

fn span_len(span: Span) -> u32 {
    span.hi().0.saturating_sub(span.lo().0)
}

fn call_def_id<'tcx>(term: &mir::Terminator<'tcx>) -> Option<DefId> {
    if let TerminatorKind::Call {
        func: Operand::Constant(fn_const),
        ..
    } = &term.kind
    {
        if let Const::Val(_, ty) = fn_const.const_ {
            if let TyKind::FnDef(def_id, _) = ty.kind() {
                return Some(*def_id);
            }
        }
    }
    None
}

fn find_best_location_for_event<'tcx>(body: &Body<'tcx>, target: Span) -> Option<Location> {
    let mut best: Option<(Location, Span)> = None;
    for (bb, bb_data) in body.basic_blocks.iter_enumerated() {
        for (idx, stmt) in bb_data.statements.iter().enumerate() {
            if matches!(
                stmt.kind,
                mir::StatementKind::StorageLive(_)
                    | mir::StatementKind::StorageDead(_)
                    | mir::StatementKind::Nop
            ) {
                continue;
            }

            let span = stmt.source_info.span;
            if spans_overlap(span, target) {
                let loc = Location {
                    block: bb,
                    statement_index: idx,
                };
                if best
                    .as_ref()
                    .is_none_or(|(_, best_span)| span_len(span) < span_len(*best_span))
                {
                    best = Some((loc, span));
                }
            }
        }

        let term = bb_data.terminator();
        let span = term.source_info.span;
        if spans_overlap(span, target) {
            let loc = body.terminator_loc(bb);
            if best
                .as_ref()
                .is_none_or(|(_, best_span)| span_len(span) < span_len(*best_span))
            {
                best = Some((loc, span));
            }
        }
    }

    best.map(|(loc, _)| loc)
}

fn map_events_to_locations<'tcx>(
    body: &Body<'tcx>,
    events: &[AuthEvent],
) -> HashMap<Location, Vec<AuthEvent>> {
    let mut map: HashMap<Location, Vec<AuthEvent>> = HashMap::new();
    for event in events {
        if let Some(loc) = find_best_location_for_event(body, event.span) {
            map.entry(loc).or_default().push(event.clone());
        }
    }
    map
}

fn map_sinks_to_locations<'tcx>(body: &Body<'tcx>, sinks: &[Sink]) -> HashMap<Location, Vec<Sink>> {
    let mut map: HashMap<Location, Vec<Sink>> = HashMap::new();
    for sink in sinks {
        if let Some(loc) = find_best_location_for_event(body, sink.span) {
            map.entry(loc).or_default().push(sink.clone());
        }
    }
    map
}

fn map_callsites_to_blocks<'tcx>(
    body: &Body<'tcx>,
    callsites: &[CallSite],
) -> HashMap<BasicBlock, CallSite> {
    let mut map = HashMap::new();

    for (bb, bb_data) in body.basic_blocks.iter_enumerated() {
        let term = bb_data.terminator();
        let Some(callee_def_id) = call_def_id(term) else {
            continue;
        };

        let term_span = term.source_info.span;
        let mut best: Option<&CallSite> = None;

        for callsite in callsites {
            if callsite.callee_def_id != callee_def_id {
                continue;
            }
            if !spans_overlap(term_span, callsite.span) {
                continue;
            }

            if best
                .as_ref()
                .is_none_or(|best_cs| span_len(callsite.span) < span_len(best_cs.span))
            {
                best = Some(callsite);
            }
        }

        if let Some(best) = best {
            map.insert(bb, best.clone());
        }
    }

    map
}

pub struct AuthFlowAnalysis<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &'a Body<'tcx>,
    param_count: usize,
    address_params: DenseBitSet<ParamIndex>,
    auth_events_by_loc: HashMap<Location, Vec<AuthEvent>>,
    callsite_by_block: HashMap<BasicBlock, CallSite>,
    fn_summaries: &'a HashMap<DefId, FnSummary>,
}

impl<'a, 'tcx> AuthFlowAnalysis<'a, 'tcx> {
    fn apply_auth_events(&self, state: &mut AuthDomain, loc: Location) {
        let Some(events) = self.auth_events_by_loc.get(&loc) else {
            return;
        };
        for event in events {
            if let Some(param_idx) = event.param_index {
                state.unauth_params.remove(ParamIndex::new(param_idx));
            }
            if event.is_current_admin {
                state.current_admin_unauth = false;
            }
        }
    }

    fn const_switch_target(
        &self,
        block: BasicBlock,
        discr: &Operand<'tcx>,
        targets: &mir::SwitchTargets,
    ) -> Option<BasicBlock> {
        let bits = match discr {
            Operand::Constant(c) => c.const_.try_eval_bits(self.tcx, self.typing_env)?,
            Operand::Copy(place) | Operand::Move(place) => {
                self.const_bits_from_place(block, place)?
            }
        };
        Some(targets.target_for_value(bits))
    }

    fn const_bits_from_place(&self, block: BasicBlock, place: &mir::Place<'tcx>) -> Option<u128> {
        if !place.projection.is_empty() {
            return None;
        }
        let local = place.local;
        let data = &self.body.basic_blocks[block];
        let mut last_const: Option<mir::Const<'tcx>> = None;

        for stmt in &data.statements {
            let mir::StatementKind::Assign(bbox) = &stmt.kind else {
                continue;
            };
            let (lhs, rvalue) = bbox.as_ref();
            if lhs.local != local || !lhs.projection.is_empty() {
                continue;
            }

            last_const = match rvalue {
                mir::Rvalue::Use(Operand::Constant(c)) => Some(c.const_),
                _ => None,
            };
        }

        last_const.and_then(|c| c.try_eval_bits(self.tcx, self.typing_env))
    }
}

impl<'tcx> Analysis<'tcx> for AuthFlowAnalysis<'_, 'tcx> {
    type Domain = AuthDomain;

    const NAME: &'static str = "missing_new_admin_auth_flow";

    fn bottom_value(&self, _body: &Body<'tcx>) -> Self::Domain {
        AuthDomain {
            unauth_params: DenseBitSet::new_empty(self.param_count),
            current_admin_unauth: false,
        }
    }

    fn initialize_start_block(&self, _body: &Body<'tcx>, state: &mut Self::Domain) {
        state.unauth_params = self.address_params.clone();
        state.current_admin_unauth = true;
    }

    fn apply_primary_statement_effect(
        &mut self,
        state: &mut Self::Domain,
        _stmt: &mir::Statement<'tcx>,
        loc: Location,
    ) {
        self.apply_auth_events(state, loc);
    }

    fn apply_primary_terminator_effect<'mir>(
        &mut self,
        state: &mut Self::Domain,
        term: &'mir mir::Terminator<'tcx>,
        loc: Location,
    ) -> mir::TerminatorEdges<'mir, 'tcx> {
        self.apply_auth_events(state, loc);

        if let TerminatorKind::SwitchInt { discr, targets } = &term.kind {
            if let Some(target) = self.const_switch_target(loc.block, discr, targets) {
                return mir::TerminatorEdges::Single(target);
            }
        }

        term.edges()
    }

    fn apply_call_return_effect(
        &mut self,
        state: &mut Self::Domain,
        block: BasicBlock,
        _return_places: mir::CallReturnPlaces<'_, 'tcx>,
    ) {
        let Some(callsite) = self.callsite_by_block.get(&block) else {
            return;
        };
        let Some(summary) = self.fn_summaries.get(&callsite.callee_def_id) else {
            return;
        };

        for (callee_idx, caller_idx) in callsite.arg_to_param.iter().enumerate() {
            if summary
                .must_auth_params
                .contains(ParamIndex::new(callee_idx))
            {
                if let Some(caller_idx) = caller_idx {
                    state.unauth_params.remove(ParamIndex::new(*caller_idx));
                }
            }
        }

        if summary.must_auth_current_admin {
            state.current_admin_unauth = false;
        }
    }
}

fn is_summary_exit<'tcx>(term: &mir::Terminator<'tcx>) -> bool {
    term.successors().next().is_none()
}

fn address_param_set(params: &[ParamInfo]) -> DenseBitSet<ParamIndex> {
    let mut set = DenseBitSet::new_empty(params.len());
    for (i, p) in params.iter().enumerate() {
        if p.is_address {
            set.insert(ParamIndex::new(i));
        }
    }
    set
}

pub fn compute_summary_for_fn<'tcx>(
    cx: &LateContext<'tcx>,
    body: &Body<'tcx>,
    params: &[ParamInfo],
    events: &[AuthEvent],
    callsites: &[CallSite],
    fn_summaries: &HashMap<DefId, FnSummary>,
) -> FnSummary {
    let address_params = address_param_set(params);
    let auth_events_by_loc = map_events_to_locations(body, events);
    let callsite_by_block = map_callsites_to_blocks(body, callsites);
    let typing_env = ty::TypingEnv::post_analysis(cx.tcx, body.source.def_id());
    let analysis = AuthFlowAnalysis {
        tcx: cx.tcx,
        typing_env,
        body,
        param_count: params.len(),
        address_params: address_params.clone(),
        auth_events_by_loc,
        callsite_by_block,
        fn_summaries,
    };

    let mut results = analysis
        .iterate_to_fixpoint(cx.tcx, body, None)
        .into_results_cursor(body);

    let mut must_auth_params = address_params;
    let mut must_auth_current_admin = true;

    for (bb, bb_data) in body.basic_blocks.iter_enumerated() {
        if !is_summary_exit(bb_data.terminator()) {
            continue;
        }
        let loc = body.terminator_loc(bb);
        results.seek_after_primary_effect(loc);
        let state = results.get();

        for p in state.unauth_params.iter() {
            must_auth_params.remove(p);
        }
        if state.current_admin_unauth {
            must_auth_current_admin = false;
        }
    }

    FnSummary {
        must_auth_params,
        must_auth_current_admin,
    }
}

pub fn lint_sinks_for_fn<'tcx>(
    cx: &LateContext<'tcx>,
    body: &Body<'tcx>,
    params: &[ParamInfo],
    events: &[AuthEvent],
    sinks: &[Sink],
    callsites: &[CallSite],
    fn_summaries: &HashMap<DefId, FnSummary>,
) {
    let address_params = address_param_set(params);
    let auth_events_by_loc = map_events_to_locations(body, events);
    let callsite_by_block = map_callsites_to_blocks(body, callsites);
    let typing_env = ty::TypingEnv::post_analysis(cx.tcx, body.source.def_id());
    let analysis = AuthFlowAnalysis {
        tcx: cx.tcx,
        typing_env,
        body,
        param_count: params.len(),
        address_params,
        auth_events_by_loc,
        callsite_by_block,
        fn_summaries,
    };

    let mut results = analysis
        .iterate_to_fixpoint(cx.tcx, body, None)
        .into_results_cursor(body);
    let sinks_by_loc = map_sinks_to_locations(body, sinks);
    for (loc, sinks_here) in sinks_by_loc {
        results.seek_before_primary_effect(loc);
        let state = results.get();

        for sink in sinks_here {
            let Some(param_idx) = sink.param_index else {
                continue;
            };
            let unauth_param = state.unauth_params.contains(ParamIndex::new(param_idx));
            let unauth_current = state.current_admin_unauth;

            if unauth_param || unauth_current {
                span_lint_and_help(
                    cx,
                    MISSING_NEW_ADMIN_AUTH,
                    sink.span,
                    LINT_MESSAGE,
                    None,
                    "require_auth must be called on both current and new admin before storing",
                );
            }
        }
    }
}

impl DebugWithContext<AuthFlowAnalysis<'_, '_>> for AuthDomain {}
