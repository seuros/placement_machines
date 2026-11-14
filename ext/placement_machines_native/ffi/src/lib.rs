//! Stateless Ruby bindings: placement algorithms live exclusively in the core.

use magnus::{Error, RClass, Ruby, Value, function, prelude::*};
use matryona_machines::ruby::{Class, Registry, Symbols, exact_array, exact_fields, integer};
use placement_machines::{Node, NodeId, Refusal, RingPoint};

/// The `Data` classes the binding builds and reads, resolved once at init.
struct Classes {
    node: Class,
    ring_point: Class,
}

static CLASSES: Registry<Classes> = Registry::new("placement_machines");

/// One sealed class as [`list`] reads it: the class, the `TypeError` for
/// anything else and its two Integer members, in order.
struct Sealed {
    class: RClass,
    message: &'static str,
    members: [&'static str; 2],
}

matryona_machines::symbols! {
    /// The core's refusals as Symbols: the snake case of each variant.
    Refusals for Refusal as "a refusal" {
        RepeatedNode => "repeated_node",
    }
}

/// The two Integer members of a value of exactly the class `sealed`, read
/// from the struct itself (no reader runs); `TypeError` for any other class,
/// even a subclass.
fn fields(ruby: &Ruby, value: Value, sealed: &Sealed) -> Result<[u64; 2], Error> {
    let [first, second] = exact_fields::<2>(ruby, value, sealed.class, sealed.message)?;
    let [first_name, second_name] = sealed.members;
    Ok([integer(first, first_name)?, integer(second, second_name)?])
}

/// Every entry of the Array `value`, each a value of exactly the class
/// `sealed` whose two Integer members `build`, the core's constructor, turns
/// into the core's value: a forged entry is re-validated, never trusted.
fn list<T>(
    ruby: &Ruby,
    value: Value,
    sealed: &Sealed,
    build: impl Fn(u64, u64) -> T,
) -> Result<Vec<T>, Error> {
    // Exactly an Array: a subclass is `TypeError` like any other subclass.
    exact_array(ruby, value, "expected an Array")?
        .into_iter()
        .map(|value| fields(ruby, value, sealed).map(|[a, b]| build(a, b)))
        .collect()
}

/// Private hook behind `Node.new`: the fields the core's `Node::new` holds.
fn node(id: Value, domain: Value) -> Result<(u64, u64), Error> {
    let node = Node::new(NodeId(integer(id, "id")?), integer(domain, "domain")?);
    Ok((node.id.0, node.domain))
}

/// Private hook behind `RingPoint.new`: the fields the core's `RingPoint::new`
/// holds.
fn ring_point(position: Value, node: Value) -> Result<(u64, u64), Error> {
    let position = integer(position, "position")?;
    let point = RingPoint::new(position, NodeId(integer(node, "node")?));
    Ok((point.position, point.node.0))
}

fn score(key: Value, node: Value) -> Result<u64, Error> {
    let key = integer(key, "key")?;
    Ok(placement_machines::score(
        key,
        NodeId(integer(node, "node")?),
    ))
}

/// The core's `replicas(key, nodes, out)`: `limit` is the length of `out`, and
/// the answer is `out` cut to the entries written. Everything that can raise
/// before the core runs does; the class was resolved at init.
fn replicas(ruby: &Ruby, key: Value, membership: Value, limit: Value) -> Result<Value, Error> {
    let (key, limit): (u64, u64) = (integer(key, "key")?, integer(limit, "limit")?);
    let class = CLASSES.get(ruby)?.node.get(ruby);
    let sealed = Sealed {
        class,
        message: "entry must be a PlacementMachines::Node",
        members: ["id", "domain"],
    };
    let nodes = list(ruby, membership, &sealed, |id, domain| {
        Node::new(NodeId(id), domain)
    })?;
    let limit = usize::try_from(limit).unwrap_or(usize::MAX);
    let mut out = vec![Node::default(); limit.min(nodes.len())];
    let written = match placement_machines::replicas(key, &nodes, &mut out) {
        Ok(written) => written,
        Err(refused) => return Ok(Refusals::symbol(ruby, refused).as_value()),
    };
    let selected = ruby.ary_new_capa(written);
    for node in &out[..written] {
        selected.push(class.funcall::<_, _, Value>("new", (node.id.0, node.domain))?)?;
    }
    Ok(selected.as_value())
}

fn ring_owner(ruby: &Ruby, key: Value, points: Value) -> Result<Option<u64>, Error> {
    let key = integer(key, "key")?;
    let sealed = Sealed {
        class: CLASSES.get(ruby)?.ring_point.get(ruby),
        message: "entry must be a PlacementMachines::RingPoint",
        members: ["position", "node"],
    };
    let ring = list(ruby, points, &sealed, |position, node| {
        RingPoint::new(position, NodeId(node))
    })?;
    Ok(placement_machines::ring_owner(key, &ring).map(|node| node.0))
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("PlacementMachines")?;
    let classes = Classes {
        node: Class::resolve(ruby, module, "Node")?,
        ring_point: Class::resolve(ruby, module, "RingPoint")?,
    };
    CLASSES.set(ruby, classes)?;
    module.define_singleton_method("__node", function!(node, 2))?;
    module.define_singleton_method("__ring_point", function!(ring_point, 2))?;
    module.define_singleton_method("score", function!(score, 2))?;
    module.define_singleton_method("replicas", function!(replicas, 3))?;
    module.define_singleton_method("ring_owner", function!(ring_owner, 2))
}
