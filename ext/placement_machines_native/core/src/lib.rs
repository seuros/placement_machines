#![no_std]
#![doc = include_str!("../README.md")]

/// Stable identity within one membership configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

/// A node and its caller-defined failure domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node {
    /// Stable identity.
    pub id: NodeId,
    /// Nodes sharing this value share one failure domain.
    pub domain: u64,
}

/// A virtual-node position in a caller-generated consistent-hash ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RingPoint {
    /// Unsigned ring coordinate.
    pub position: u64,
    /// Owner of this virtual point.
    pub node: NodeId,
}

/// Deterministic noncryptographic rendezvous score (larger wins).
#[must_use]
pub const fn score(key: u64, node: NodeId) -> u64 {
    let mut x = key.wrapping_add(node.0.wrapping_mul(0x9e37_79b9_7f4a_7c15));
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// Rank unique node IDs into distinct-domain replica slots.
///
/// Duplicate IDs reject the membership. Insufficient domains leave `None` slots.
#[must_use]
pub fn replicas<const R: usize>(key: u64, nodes: &[Node]) -> Option<[Option<Node>; R]> {
    for (index, node) in nodes.iter().enumerate() {
        if nodes[..index].iter().any(|other| other.id == node.id) {
            return None;
        }
    }
    let mut selected = [None; R];
    for slot in 0..R {
        selected[slot] = nodes
            .iter()
            .copied()
            .filter(|node| {
                !selected[..slot]
                    .iter()
                    .flatten()
                    .any(|chosen: &Node| chosen.domain == node.domain)
            })
            .max_by(|left, right| {
                score(key, left.id)
                    .cmp(&score(key, right.id))
                    .then_with(|| right.id.cmp(&left.id))
            });
    }
    Some(selected)
}

/// Clockwise consistent-hash owner; supports unsorted virtual points and wraparound.
#[must_use]
pub fn ring_owner(key: u64, ring: &[RingPoint]) -> Option<NodeId> {
    ring.iter()
        .min_by_key(|point| (point.position.wrapping_sub(key), point.node))
        .map(|point| point.node)
}

#[cfg(test)]
mod tests;
