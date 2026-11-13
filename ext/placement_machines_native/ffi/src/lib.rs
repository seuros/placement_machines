//! Stateless Ruby bindings: placement algorithms live exclusively in the core.

use magnus::{Error, Integer, RArray, RClass, Ruby, TryConvert, Value, function, prelude::*};
use placement_machines::{Node, NodeId, Refusal, RingPoint};

/// Most nodes one `replicas` call reads.
const MAX_NODES: usize = 64;
/// Most ring points one `ring_owner` call reads.
const MAX_RING_POINTS: usize = 1024;

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

/// `PlacementMachines::Node` or `PlacementMachines::RingPoint`.
fn data_class(ruby: &Ruby, name: &str) -> Result<RClass, Error> {
    ruby.define_module("PlacementMachines")?.const_get(name)
}

fn array(ruby: &Ruby, value: Value, maximum: usize) -> Result<RArray, Error> {
    let array = RArray::from_value(value)
        .ok_or_else(|| Error::new(ruby.exception_type_error(), "expected an Array"))?;
    if array.len() > maximum {
        return Err(Error::new(
            ruby.exception_range_error(),
            format!("at most {maximum} entries are supported"),
        ));
    }
    Ok(array)
}

/// The two Integer fields of a `Data` value of exactly the class `name`.
fn fields(ruby: &Ruby, value: Value, name: &str, readers: [&str; 2]) -> Result<[u64; 2], Error> {
    if !value.is_kind_of(data_class(ruby, name)?) {
        return Err(Error::new(
            ruby.exception_type_error(),
            format!("expected a PlacementMachines::{name}"),
        ));
    }
    let first: Number = value.funcall(readers[0], ())?;
    let second: Number = value.funcall(readers[1], ())?;
    Ok([first.0, second.0])
}

fn nodes(ruby: &Ruby, value: Value) -> Result<Vec<Node>, Error> {
    array(ruby, value, MAX_NODES)?
        .into_iter()
        .map(|value| {
            let [id, domain] = fields(ruby, value, "Node", ["id", "domain"])?;
            Ok(Node {
                id: NodeId(id),
                domain,
            })
        })
        .collect()
}

const fn score(key: Number, node: Number) -> u64 {
    placement_machines::score(key.0, NodeId(node.0))
}

/// The core's `replicas(key, nodes, out)`: `limit` is the length of `out`, and
/// the answer is `out` cut to the entries written.
fn replicas(ruby: &Ruby, key: Number, membership: Value, limit: Number) -> Result<Value, Error> {
    let nodes = nodes(ruby, membership)?;
    let limit = usize::try_from(limit.0).unwrap_or(usize::MAX);
    let mut out = vec![Node::default(); limit.min(nodes.len())];
    let written = match placement_machines::replicas(key.0, &nodes, &mut out) {
        Ok(written) => written,
        Err(refused) => return Ok(ruby.to_symbol(refusal(refused)).as_value()),
    };
    let class = data_class(ruby, "Node")?;
    let selected = ruby.ary_new_capa(written);
    for node in &out[..written] {
        selected.push(class.funcall::<_, _, Value>("new", (node.id.0, node.domain))?)?;
    }
    Ok(selected.as_value())
}

fn ring_owner(ruby: &Ruby, key: Number, points: Value) -> Result<Option<u64>, Error> {
    let ring = array(ruby, points, MAX_RING_POINTS)?
        .into_iter()
        .map(|value| {
            let [position, node] = fields(ruby, value, "RingPoint", ["position", "node"])?;
            Ok(RingPoint {
                position,
                node: NodeId(node),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(placement_machines::ring_owner(key.0, &ring).map(|node| node.0))
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("PlacementMachines")?;
    module.const_set("MAX_NODES", MAX_NODES)?;
    module.const_set("MAX_RING_POINTS", MAX_RING_POINTS)?;
    module.define_singleton_method("score", function!(score, 2))?;
    module.define_singleton_method("replicas", function!(replicas, 3))?;
    module.define_singleton_method("ring_owner", function!(ring_owner, 2))
}
