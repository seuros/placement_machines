#![no_std]
#![doc = include_str!("../README.md")]

use core::fmt;

/// Stable identity within one membership configuration, chosen by the caller.
///
/// `NodeId::default()` is `NodeId(0)`: a placeholder for filling an output
/// buffer, and an ordinary id when the caller uses zero.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

/// A node and its caller-defined failure domain.
///
/// `Node::default()` is a placeholder for filling an output buffer, not a
/// sentinel: read only the entries a call reported.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Node {
    /// Stable identity.
    pub id: NodeId,
    /// Nodes sharing this value share one failure domain.
    pub domain: u64,
}

/// A virtual-node position in a caller-generated consistent-hash ring.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RingPoint {
    /// Unsigned ring coordinate.
    pub position: u64,
    /// Owner of this virtual point.
    pub node: NodeId,
}

/// Why a call answered nothing and wrote nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Two entries of the membership carry the same [`NodeId`].
    RepeatedNode,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RepeatedNode => "two entries of the membership carry the same node id",
        })
    }
}

impl core::error::Error for Refusal {}

/// Deterministic noncryptographic rendezvous score (larger wins).
#[must_use]
pub const fn score(key: u64, node: NodeId) -> u64 {
    let mut x = key.wrapping_add(node.0.wrapping_mul(0x9e37_79b9_7f4a_7c15));
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// Rank unique node IDs into distinct-domain replicas, best first.
///
/// Writes the first `n` entries of `out`, leaves the rest untouched and
/// returns `n`: at most `out.len()`, and fewer when the membership has fewer
/// failure domains. The first entry is the rendezvous owner of `key`.
///
/// # Errors
///
/// [`Refusal::RepeatedNode`] when two entries of `nodes` share an id; `out`
/// is untouched.
pub fn replicas(key: u64, nodes: &[Node], out: &mut [Node]) -> Result<usize, Refusal> {
    for (index, node) in nodes.iter().enumerate() {
        if nodes[..index].iter().any(|other| other.id == node.id) {
            return Err(Refusal::RepeatedNode);
        }
    }
    let mut written = 0;
    while written < out.len() {
        let chosen = &out[..written];
        let best = nodes
            .iter()
            .copied()
            .filter(|node| !chosen.iter().any(|chosen| chosen.domain == node.domain))
            .max_by(|left, right| {
                score(key, left.id)
                    .cmp(&score(key, right.id))
                    .then_with(|| right.id.cmp(&left.id))
            });
        let Some(best) = best else {
            break;
        };
        out[written] = best;
        written += 1;
    }
    Ok(written)
}

/// Clockwise consistent-hash owner; supports unsorted virtual points and wraparound.
///
/// `None` only for an empty ring.
#[must_use]
pub fn ring_owner(key: u64, ring: &[RingPoint]) -> Option<NodeId> {
    ring.iter()
        .min_by_key(|point| (point.position.wrapping_sub(key), point.node))
        .map(|point| point.node)
}

#[cfg(test)]
mod tests;
