//! Randomized comparison against sort-based references.
extern crate std;

use crate::{Node, NodeId, Refusal, RingPoint, replicas, ring_owner, score};
use std::vec::Vec;

/// `SplitMix64`: deterministic draws, no dependency.
struct Draws(u64);

impl Draws {
    const fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut x = self.0;
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^ (x >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % (bound as u64)).expect("fits")
    }
}

const UNTOUCHED: Node = Node {
    id: NodeId(u64::MAX),
    domain: u64::MAX,
};

/// Rank everything, then keep the first node of each domain.
fn reference(key: u64, nodes: &[Node], limit: usize) -> Option<Vec<Node>> {
    let mut ranked = nodes.to_vec();
    ranked.sort_by_key(|node| (core::cmp::Reverse(score(key, node.id)), node.id));
    if ranked.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return None;
    }
    let mut chosen: Vec<Node> = Vec::new();
    for node in ranked {
        if chosen.len() < limit && chosen.iter().all(|other| other.domain != node.domain) {
            chosen.push(node);
        }
    }
    Some(chosen)
}

#[test]
fn random_memberships_match_the_ranked_reference() {
    let mut draws = Draws(4213);
    let (mut refused, mut short) = (0, 0);
    for _ in 0..4_000 {
        let nodes: Vec<Node> = (0..draws.below(9))
            .map(|_| Node {
                id: NodeId(draws.next() % 24),
                domain: draws.next() % 4,
            })
            .collect();
        let key = draws.next();
        let limit = draws.below(7);
        let mut out = [UNTOUCHED; 8];
        let before = out;
        let answer = replicas(key, &nodes, &mut out[..limit]);
        match reference(key, &nodes, limit) {
            None => {
                refused += 1;
                assert_eq!(answer, Err(Refusal::RepeatedNode));
                assert_eq!(out, before);
            }
            Some(expected) => {
                short += usize::from(expected.len() < limit);
                assert_eq!(answer, Ok(expected.len()));
                assert_eq!(out[..expected.len()], expected[..]);
                assert_eq!(out[expected.len()..], before[expected.len()..]);
                // Asking again is the same answer: the functions keep no state.
                let mut again = [UNTOUCHED; 8];
                assert_eq!(replicas(key, &nodes, &mut again[..limit]), answer);
                assert_eq!(again, out);
            }
        }
    }
    assert!(refused > 100 && short > 100);
}

#[test]
fn random_rings_match_the_clockwise_reference() {
    let mut draws = Draws(1997);
    for _ in 0..2_000 {
        let ring: Vec<RingPoint> = (0..draws.below(8))
            .map(|_| RingPoint {
                position: draws.next() % 64,
                node: NodeId(draws.next() % 5),
            })
            .collect();
        let key = draws.next() % 80;
        // Clockwise distance in wide arithmetic, nearest first, lowest id on a tie.
        let distance = |point: &RingPoint| {
            (u128::from(point.position) + (1 << 64) - u128::from(key)) % (1 << 64)
        };
        let nearest = ring.iter().map(distance).min();
        let expected = ring
            .iter()
            .filter(|point| Some(distance(point)) == nearest)
            .map(|point| point.node)
            .min();
        assert_eq!(ring_owner(key, &ring), expected);
    }
}
