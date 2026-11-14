#![allow(clippy::expect_used)]
mod model;

use super::*;

const NODES: [Node; 3] = [node(1, 1), node(2, 2), node(3, 3)];

/// A buffer entry no call below can write: proves untouched slots.
const UNTOUCHED: Node = node(u64::MAX, u64::MAX);

const fn node(id: u64, domain: u64) -> Node {
    Node::new(NodeId(id), domain)
}

fn owner(key: u64, nodes: &[Node]) -> Option<Node> {
    let mut out = [Node::default(); 1];
    let written = replicas(key, nodes, &mut out).expect("unique nodes");
    (written == 1).then_some(out[0])
}

#[test]
fn rendezvous_is_order_independent_and_removal_only_moves_owned_keys() {
    let reversed = [NODES[2], NODES[1], NODES[0]];
    for key in 0..1_000 {
        let first = owner(key, &NODES);
        assert!(first.is_some());
        assert_eq!(owner(key, &reversed), first);
        if first != Some(NODES[2]) {
            assert_eq!(first, owner(key, &NODES[..2]));
        }
    }
}

#[test]
fn replicas_never_repeat_a_domain_or_hide_shortfalls() {
    let nodes = [NODES[0], node(2, 1), NODES[2]];
    let mut out = [UNTOUCHED; 3];
    assert_eq!(replicas(9, &nodes, &mut out), Ok(2));
    assert_ne!(out[0].domain, out[1].domain);
    assert!(nodes.contains(&out[0]) && nodes.contains(&out[1]));
    assert_eq!(out[2], UNTOUCHED);
    assert_eq!(Some(out[0]), owner(9, &nodes));
}

#[test]
fn the_buffer_bounds_the_answer_and_the_rest_is_untouched() {
    let mut out = [UNTOUCHED; 5];
    assert_eq!(replicas(7, &NODES, &mut out[..2]), Ok(2));
    assert_eq!(out[2..], [UNTOUCHED; 3]);
    let first_two = [out[0], out[1]];
    assert_eq!(replicas(7, &NODES, &mut out), Ok(3));
    assert_eq!(out[..2], first_two);
    assert_eq!(out[3..], [UNTOUCHED; 2]);
    assert_eq!(replicas(7, &NODES, &mut []), Ok(0));
    assert_eq!(replicas(7, &[], &mut out), Ok(0));
    assert_eq!(owner(7, &[]), None);
}

#[test]
fn a_repeated_node_id_is_refused_and_writes_nothing() {
    let mut out = [UNTOUCHED; 3];
    let before = out;
    for nodes in [
        &[NODES[0], NODES[0]][..],
        &[NODES[0], NODES[1], node(1, 9)],
        &[node(0, 1), node(0, 2)],
    ] {
        assert_eq!(replicas(0, nodes, &mut out), Err(Refusal::RepeatedNode));
        assert_eq!(out, before);
        assert_eq!(replicas(0, nodes, &mut []), Err(Refusal::RepeatedNode));
    }
}

#[test]
fn ring_lookup_wraps_and_breaks_ties_stably() {
    let point = |position, node| RingPoint::new(position, NodeId(node));
    let ring = [point(10, 2), point(30, 1)];
    assert_eq!(ring_owner(11, &ring), Some(NodeId(1)));
    assert_eq!(ring_owner(31, &ring), Some(NodeId(2)));
    assert_eq!(ring_owner(10, &ring), Some(NodeId(2)));
    assert_eq!(
        ring_owner(10, &[point(10, 2), point(10, 0)]),
        Some(NodeId(0))
    );
    assert_eq!(ring_owner(0, &[]), None);
}

#[test]
fn placeholders_and_refusals_describe_themselves() {
    extern crate std;
    use std::string::ToString;
    assert_eq!(Node::default(), node(0, 0));
    assert_eq!(RingPoint::default(), RingPoint::new(0, NodeId(0)));
    let error: &dyn core::error::Error = &Refusal::RepeatedNode;
    assert_ne!(error.to_string(), "");
}
