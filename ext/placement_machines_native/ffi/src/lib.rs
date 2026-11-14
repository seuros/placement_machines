//! Stateless Ruby bindings: placement algorithms live exclusively in the core.

mod sealed;

use magnus::{Error, Integer, RArray, Ruby, TryConvert, Value, function, prelude::*};
use placement_machines::{Node, NodeId, Refusal, RingPoint};
use sealed::Sealed;

/// `PlacementMachines::Node`, resolved at init.
static NODE: Sealed = Sealed::new("Node");
/// `PlacementMachines::RingPoint`, resolved at init.
static RING_POINT: Sealed = Sealed::new("RingPoint");

#[derive(Clone, Copy)]
struct Number(u64);

impl TryConvert for Number {
    fn try_convert(value: Value) -> Result<Self, Error> {
        if Integer::from_value(value).is_none() {
            return Err(Error::new(
                Ruby::get_with(value).exception_type_error(),
                "keys, IDs, domains, positions and limits must be Integers",
            ));
        }
        u64::try_convert(value).map(Self)
    }
}

const fn refusal(refusal: Refusal) -> &'static str {
    match refusal {
        Refusal::RepeatedNode => "repeated_node",
    }
}

fn array(ruby: &Ruby, value: Value) -> Result<RArray, Error> {
    RArray::from_value(value)
        .ok_or_else(|| Error::new(ruby.exception_type_error(), "expected an Array"))
}

/// The two Integer members of a value of exactly the class `sealed`, read
/// natively; `TypeError` for any other class, even a subclass.
fn fields(
    ruby: &Ruby,
    value: Value,
    sealed: &Sealed,
    members: [&str; 2],
) -> Result<[u64; 2], Error> {
    let read = sealed.read(ruby, value, "entry")?;
    let first: Number = read.get(members[0])?;
    let second: Number = read.get(members[1])?;
    Ok([first.0, second.0])
}

/// Every entry of the Array `value`, each a value of exactly the class
/// `sealed` whose two Integer members `build`, the core's constructor, turns
/// into the core's value: a forged entry is re-validated, never trusted.
fn list<T>(
    ruby: &Ruby,
    value: Value,
    sealed: &Sealed,
    members: [&str; 2],
    build: impl Fn(u64, u64) -> T,
) -> Result<Vec<T>, Error> {
    array(ruby, value)?
        .into_iter()
        .map(|value| fields(ruby, value, sealed, members).map(|[a, b]| build(a, b)))
        .collect()
}

/// Private hook behind `Node.new`: the fields the core's `Node::new` holds.
const fn node(id: Number, domain: Number) -> (u64, u64) {
    let node = Node::new(NodeId(id.0), domain.0);
    (node.id.0, node.domain)
}

/// Private hook behind `RingPoint.new`: the fields the core's `RingPoint::new`
/// holds.
const fn ring_point(position: Number, node: Number) -> (u64, u64) {
    let point = RingPoint::new(position.0, NodeId(node.0));
    (point.position, point.node.0)
}

const fn score(key: Number, node: Number) -> u64 {
    placement_machines::score(key.0, NodeId(node.0))
}

/// The core's `replicas(key, nodes, out)`: `limit` is the length of `out`, and
/// the answer is `out` cut to the entries written. Everything that can raise
/// before the core runs does; the class was resolved at init.
fn replicas(ruby: &Ruby, key: Number, membership: Value, limit: Number) -> Result<Value, Error> {
    let nodes = list(ruby, membership, &NODE, ["id", "domain"], |id, domain| {
        Node::new(NodeId(id), domain)
    })?;
    let limit = usize::try_from(limit.0).unwrap_or(usize::MAX);
    let mut out = vec![Node::default(); limit.min(nodes.len())];
    let written = match placement_machines::replicas(key.0, &nodes, &mut out) {
        Ok(written) => written,
        Err(refused) => return Ok(ruby.to_symbol(refusal(refused)).as_value()),
    };
    let selected = ruby.ary_new_capa(written);
    for node in &out[..written] {
        selected.push(NODE.build(ruby, (node.id.0, node.domain))?)?;
    }
    Ok(selected.as_value())
}

fn ring_owner(ruby: &Ruby, key: Number, points: Value) -> Result<Option<u64>, Error> {
    let ring = list(
        ruby,
        points,
        &RING_POINT,
        ["position", "node"],
        |position, node| RingPoint::new(position, NodeId(node)),
    )?;
    Ok(placement_machines::ring_owner(key.0, &ring).map(|node| node.0))
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("PlacementMachines")?;
    NODE.resolve(ruby, module)?;
    RING_POINT.resolve(ruby, module)?;
    module.define_singleton_method("__node", function!(node, 2))?;
    module.define_singleton_method("__ring_point", function!(ring_point, 2))?;
    module.define_singleton_method("score", function!(score, 2))?;
    module.define_singleton_method("replicas", function!(replicas, 3))?;
    module.define_singleton_method("ring_owner", function!(ring_owner, 2))
}
