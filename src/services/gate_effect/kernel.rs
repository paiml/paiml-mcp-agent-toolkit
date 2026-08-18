//! The pure kernels CB-2100 rests on.
//!
//! Everything else in this module tree reads YAML, shells out, or touches the
//! filesystem. These three functions do none of that: they are total, they have
//! no inputs beyond their arguments, and they are the only place where the two
//! decisions that make or break the rule are actually taken.
//!
//! * [`reachable`] — is there a path from **any** required-check root to this
//!   invocation along edges whose failure can still propagate? (INV-2100-1,
//!   INV-2100-2.)
//! * [`select_by_context`] — which candidate does a required context string
//!   name? Matching is on the *context*; the display name is carried into the
//!   function precisely so that a proof can show it is never read. (INV-2100-3.)
//! * [`gates`] — a command that printed a failure verdict and exited 0 did not
//!   gate anything. (INV-2100-4.)
//!
//! Kani harnesses live at the bottom of the file under `#[cfg(kani)]`.

/// One edge of the enforcement graph.
///
/// `live` is the whole point: an edge exists in the YAML whether or not a
/// failure can cross it, and CB-2100's second invariant is that a dead edge is
/// not an edge at all. Carrying it as data (rather than filtering edges out at
/// construction time) keeps the "why" available for the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    /// `false` ⇒ a failure at `to` cannot reach `from`.
    pub live: bool,
}

impl Edge {
    pub const fn live(from: usize, to: usize) -> Self {
        Self {
            from,
            to,
            live: true,
        }
    }

    pub const fn dead(from: usize, to: usize) -> Self {
        Self {
            from,
            to,
            live: false,
        }
    }
}

/// Is `target` reachable from **any** of `roots` along live edges only?
///
/// Fails closed: an out-of-range target is not reachable, an out-of-range root
/// contributes nothing, and an empty root set makes everything unreachable.
/// "We could not place this node in the graph" must never read as "enforced".
#[provable_contracts_macros::contract("comply-gate-effect-v1.yaml", equation = "reachable")]
pub fn reachable(node_count: usize, edges: &[Edge], roots: &[usize], target: usize) -> bool {
    if target >= node_count {
        return false;
    }
    let mut seen = vec![false; node_count];
    let mut stack: Vec<usize> = Vec::new();
    for &r in roots {
        if r < node_count && !seen[r] {
            seen[r] = true;
            stack.push(r);
        }
    }
    while let Some(n) = stack.pop() {
        for e in edges {
            if e.from == n && e.live && e.to < node_count && !seen[e.to] {
                seen[e.to] = true;
                stack.push(e.to);
            }
        }
    }
    seen[target]
}

/// The index of the first candidate whose **context** equals `required`.
///
/// Each candidate is `(context, display_name)`. The display name is passed in
/// and deliberately never read: INV-2100-3 is exactly the claim that a job
/// whose *display name* happens to equal a required context string is not the
/// job that reports it. Keeping `display` in the signature is what lets
/// `KANI-2100-2` prove the claim instead of asserting it in prose.
#[provable_contracts_macros::contract("comply-gate-effect-v1.yaml", equation = "select_by_context")]
pub fn select_by_context<K: PartialEq>(candidates: &[(K, K)], required: &K) -> Option<usize> {
    candidates
        .iter()
        .position(|(context, _display)| context == required)
}

/// Did a command that printed a failure verdict actually gate?
///
/// The whole of INV-2100-4 in one line: printing `FAILED` and exiting 0 is
/// theatre. A command that printed nothing damning gates by definition (there
/// was no verdict to contradict), and any non-zero exit gates.
#[provable_contracts_macros::contract("comply-gate-effect-v1.yaml", equation = "gates")]
pub const fn gates(printed_failure_verdict: bool, exit_code: i32) -> bool {
    !printed_failure_verdict || exit_code != 0
}

// ── Kani ────────────────────────────────────────────────────────────────────

#[cfg(kani)]
mod proofs {
    use super::*;

    const N: usize = 4;
    const E: usize = 6;

    /// Reference implementation: transitive closure by repeated relaxation.
    /// Deliberately a different algorithm from [`reachable`]'s stack walk — two
    /// spellings of the same bug would prove nothing.
    fn closure_reachable(edges: &[Edge; E], roots: &[usize], target: usize) -> bool {
        let mut seen = [false; N];
        for &r in roots {
            if r < N {
                seen[r] = true;
            }
        }
        // N rounds saturate any path of length < N over N nodes.
        for _ in 0..N {
            for e in edges.iter() {
                if e.live && e.from < N && e.to < N && seen[e.from] {
                    seen[e.to] = true;
                }
            }
        }
        seen[target]
    }

    fn any_edges() -> [Edge; E] {
        let mut edges = [Edge::live(0, 0); E];
        for e in edges.iter_mut() {
            let from: usize = kani::any();
            let to: usize = kani::any();
            kani::assume(from < N && to < N);
            *e = Edge {
                from,
                to,
                live: kani::any(),
            };
        }
        edges
    }

    /// `KANI-2100-1a`: `reachable` agrees with an independent transitive
    /// closure over a bounded graph — reachable ⇔ ∃ a path of live edges from
    /// some root.
    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_reachable_matches_transitive_closure() {
        let edges = any_edges();
        let r0: usize = kani::any();
        let r1: usize = kani::any();
        kani::assume(r0 < N && r1 < N);
        let roots = [r0, r1];
        let target: usize = kani::any();
        kani::assume(target < N);

        assert_eq!(
            reachable(N, &edges, &roots, target),
            closure_reachable(&edges, &roots, target)
        );
    }

    /// `KANI-2100-1b`: neuter every edge and nothing but a root is reachable.
    /// This is INV-2100-2 as a theorem: if no failure can cross any edge, the
    /// only "enforced" nodes are the required checks themselves.
    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_neutering_every_edge_kills_reachability() {
        let mut edges = any_edges();
        for e in edges.iter_mut() {
            e.live = false;
        }
        let r0: usize = kani::any();
        kani::assume(r0 < N);
        let roots = [r0];
        let target: usize = kani::any();
        kani::assume(target < N);

        assert_eq!(reachable(N, &edges, &roots, target), target == r0);
    }

    /// `KANI-2100-1c`: an empty root set reaches nothing. Fail-closed as a
    /// theorem — "no roots" must never collapse into "everything is fine".
    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_no_roots_reaches_nothing() {
        let edges = any_edges();
        let target: usize = kani::any();
        kani::assume(target < N);
        assert!(!reachable(N, &edges, &[], target));
    }

    /// `KANI-2100-2`: the context matcher reads the context and never the
    /// display name. Two candidate lists differing only in their display names
    /// select the same candidate, and a selected candidate's *context* always
    /// equals the required string — so a display name equal to a required
    /// context can never, on its own, satisfy reachability.
    #[kani::proof]
    #[kani::unwind(6)]
    fn verify_select_ignores_display_names() {
        let c0: u8 = kani::any();
        let c1: u8 = kani::any();
        let c2: u8 = kani::any();
        let d0: u8 = kani::any();
        let d1: u8 = kani::any();
        let d2: u8 = kani::any();
        let e0: u8 = kani::any();
        let e1: u8 = kani::any();
        let e2: u8 = kani::any();
        let required: u8 = kani::any();

        let a = [(c0, d0), (c1, d1), (c2, d2)];
        let b = [(c0, e0), (c1, e1), (c2, e2)];

        let sel = select_by_context(&a, &required);
        assert_eq!(sel, select_by_context(&b, &required));

        match sel {
            Some(i) => assert!(a[i].0 == required),
            // Not selected ⇒ no candidate's CONTEXT matched, whatever any
            // display name happened to be.
            None => assert!(c0 != required && c1 != required && c2 != required),
        }
    }

    /// `KANI-2100-4`: printing a failure verdict and exiting 0 never gates, and
    /// nothing else is ever called ungated.
    #[kani::proof]
    fn verify_gates_only_on_exit_code() {
        let printed: bool = kani::any();
        let code: i32 = kani::any();
        assert_eq!(gates(printed, code), !printed || code != 0);
        assert!(!gates(true, 0));
        assert!(gates(false, 0));
    }
}
