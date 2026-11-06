# placement-machines

> **AMS Karger**, NCC-4213, *Karger*-class navigation tender.

Engine-room flight manual: chart deterministic destinations and replica berths.

Allocation-free `no_std` placement over stable caller-supplied numeric key hashes
and node identities. Rendezvous scores use a documented SplitMix64-style mixer,
not a cryptographic hash. Do not use these scores for adversarial admission or
security decisions. Ties favor lower node IDs, independent of input order.

Replica selection requires unique node IDs and chooses at most one node in each
failure domain. Missing replicas are explicit `None` values, never repeated nodes.
Weights, fallback within an exhausted domain, and topology hierarchies are not
implemented. Ring lookup accepts caller-generated virtual points, including
wraparound; ring construction and hash-version agreement are external contracts.
Placement does not define replica membership changes or consensus quorums.

```rust
use placement_machines::{Node, NodeId, replicas};
let nodes = [
    Node { id: NodeId(1), domain: 10 },
    Node { id: NodeId(2), domain: 20 },
];
# fn example(nodes: &[Node]) -> Option<()> {
let selected = replicas::<2>(42, nodes)?;
assert!(selected[0].is_some());
assert!(selected[1].is_some());
assert_ne!(selected[0], selected[1]);
# Some(())
# }
# assert_eq!(example(&nodes), Some(()));
```
