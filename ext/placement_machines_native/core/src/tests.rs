#![allow(clippy::expect_used)]
use super::*;

const NODES: [Node; 3] = [
    Node {
        id: NodeId(1),
        domain: 1,
    },
    Node {
        id: NodeId(2),
        domain: 2,
    },
    Node {
        id: NodeId(3),
        domain: 3,
    },
];

#[test]
fn rendezvous_is_order_independent_and_removal_only_moves_owned_keys() {
    let reversed = [NODES[2], NODES[1], NODES[0]];
    for key in 0..1_000 {
        let first = replicas::<1>(key, &NODES).expect("unique nodes")[0];
        assert_eq!(
            replicas::<1>(key, &reversed).expect("unique nodes")[0],
            first
        );
        let reduced = replicas::<1>(key, &NODES[..2]).expect("unique nodes")[0];
        if first != Some(NODES[2]) {
            assert_eq!(first, reduced);
        }
    }
}

#[test]
fn replicas_never_repeat_a_domain_or_hide_shortfalls() {
    let nodes = [
        NODES[0],
        Node {
            domain: 1,
            ..NODES[1]
        },
        NODES[2],
    ];
    let selected = replicas::<3>(9, &nodes).expect("unique nodes");
    assert!(selected[0].is_some());
    assert!(selected[1].is_some());
    assert_eq!(selected[2], None);
    assert_ne!(
        selected[0].expect("first").domain,
        selected[1].expect("second").domain
    );
    assert!(replicas::<1>(0, &[NODES[0], NODES[0]]).is_none());
    assert_eq!(replicas::<0>(0, &[]), Some([]));
}

#[test]
fn ring_lookup_wraps_and_breaks_ties_stably() {
    let ring = [
        RingPoint {
            position: 10,
            node: NodeId(2),
        },
        RingPoint {
            position: 30,
            node: NodeId(1),
        },
    ];
    assert_eq!(ring_owner(11, &ring), Some(NodeId(1)));
    assert_eq!(ring_owner(31, &ring), Some(NodeId(2)));
    assert_eq!(ring_owner(10, &ring), Some(NodeId(2)));
    assert_eq!(ring_owner(0, &[]), None);
}
