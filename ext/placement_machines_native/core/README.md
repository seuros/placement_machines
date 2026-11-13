# placement-machines

> **AMS Karger**, NCC-4213, *Karger*-class navigation tender.

Engine-room flight manual: chart deterministic destinations and replica berths.

Allocation-free `no_std` placement over stable caller-supplied numeric key hashes
and node identities. Rendezvous scores use a documented SplitMix64-style mixer,
not a cryptographic hash. Do not use these scores for adversarial admission or
security decisions. Ties favor lower node IDs, independent of input order.

Replica selection requires unique node IDs and chooses at most one node in each
failure domain. Missing replicas are a shorter answer, never repeated nodes.
Weights, fallback within an exhausted domain, and topology hierarchies are not
implemented. Ring lookup accepts caller-generated virtual points, including
wraparound; ring construction and hash-version agreement are external contracts.
Placement does not define replica membership changes or consensus quorums.

```rust
use placement_machines::{Node, NodeId, Refusal, replicas};
let nodes = [
    Node { id: NodeId(1), domain: 10 },
    Node { id: NodeId(2), domain: 20 },
];
let mut out = [Node::default(); 3];
let written = replicas(42, &nodes, &mut out)?;
// Two failure domains: two replicas, and the third entry was never written.
assert_eq!(written, 2);
assert_ne!(out[0].domain, out[1].domain);

let repeated = [nodes[0], nodes[0]];
assert_eq!(replicas(42, &repeated, &mut out), Err(Refusal::RepeatedNode));
# Ok::<(), Refusal>(())
```

## Contract

- **Stateless.** Three pure functions: `score`, `replicas`, `ring_owner`.
  Nothing is stored between calls, so there is nothing to repeat, restore or
  scope; the same inputs always give the same answer.
- **Output buffer.** `replicas(key, nodes, out: &mut [Node])` writes the first
  `n` entries of `out`, best first, leaves the rest untouched and returns
  `Ok(n)`. `n` is at most `out.len()` and is smaller when the membership has
  fewer failure domains; `Ok(0)` for an empty membership or an empty buffer.
  `out[0]` is the rendezvous owner of the key.
- **Refusals.** `Refusal` is the one refusal type and has one variant:
  `RepeatedNode`, two entries of `nodes` carry the same `NodeId` (whatever
  their domains). The membership is rejected whole and `out` is untouched,
  whatever its length. `score` and `ring_owner` cannot refuse.
- **Placeholders.** `Node`, `NodeId` and `RingPoint` implement `Default` so a
  buffer is `[Node::default(); N]`. The default is a placeholder, not a
  sentinel: `NodeId(0)` is an ordinary id a caller may use. Read only the `n`
  entries reported.
- **Ids are the caller's.** `NodeId(pub u64)` is supplied by the caller and
  built directly; nothing here issues ids, so there are no scopes.
- **Queries.** `ring_owner` answers `None` only for an empty ring. Two points
  on one position go to the lower node id.
- **No time.** Keys, ids, domains and ring positions are plain `u64` values;
  no function reads or takes a clock.
