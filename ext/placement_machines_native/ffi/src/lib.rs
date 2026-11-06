//! Stateless Ruby bindings: placement algorithms live exclusively in the core.

use magnus::{Error, Integer, RArray, Ruby, TryConvert, Value, function, prelude::*};
use placement_machines::{Node, NodeId, RingPoint};

const REPLICA_COUNT: usize = 3;
const MAX_NODES: usize = 64;
const MAX_RING_POINTS: usize = 1024;

#[derive(Clone, Copy)]
struct Number(u64);

impl TryConvert for Number {
    fn try_convert(value: Value) -> Result<Self, Error> {
        if Integer::from_value(value).is_none() {
            return Err(Error::new(
                Ruby::get_with(value).exception_type_error(),
                "keys, IDs, domains and positions must be Integers",
            ));
        }
        u64::try_convert(value).map(Self)
    }
}

fn array(ruby: &Ruby, value: Value, maximum: usize) -> Result<RArray, Error> {
    let array = RArray::from_value(value)
        .ok_or_else(|| Error::new(ruby.exception_type_error(), "expected an Array"))?;
    if array.len() > maximum {
        return Err(Error::new(
            ruby.exception_arg_error(),
            format!("at most {maximum} entries are supported"),
        ));
    }
    Ok(array)
}

fn pair(ruby: &Ruby, value: Value) -> Result<(u64, u64), Error> {
    let pair = array(ruby, value, 2)?;
    if pair.len() != 2 {
        return Err(Error::new(
            ruby.exception_arg_error(),
            "each entry must contain exactly two Integers",
        ));
    }
    let [first, second] = pair.to_array::<Number, 2>()?;
    Ok((first.0, second.0))
}

fn nodes(ruby: &Ruby, value: Value) -> Result<Vec<Node>, Error> {
    array(ruby, value, MAX_NODES)?
        .into_iter()
        .map(|value| {
            let (id, domain) = pair(ruby, value)?;
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

fn replicas(ruby: &Ruby, key: Number, membership: Value) -> Result<Vec<Option<u64>>, Error> {
    let nodes = nodes(ruby, membership)?;
    let selected = placement_machines::replicas::<REPLICA_COUNT>(key.0, &nodes)
        .ok_or_else(|| Error::new(ruby.exception_arg_error(), "duplicate node IDs"))?;
    Ok(selected
        .into_iter()
        .map(|node| node.map(|node| node.id.0))
        .collect())
}

fn rendezvous_owner(ruby: &Ruby, key: Number, membership: Value) -> Result<Option<u64>, Error> {
    let nodes = nodes(ruby, membership)?;
    let selected = placement_machines::replicas::<1>(key.0, &nodes)
        .ok_or_else(|| Error::new(ruby.exception_arg_error(), "duplicate node IDs"))?;
    Ok(selected[0].map(|node| node.id.0))
}

fn ring_owner(ruby: &Ruby, key: Number, points: Value) -> Result<Option<u64>, Error> {
    let ring = array(ruby, points, MAX_RING_POINTS)?
        .into_iter()
        .map(|value| {
            let (position, id) = pair(ruby, value)?;
            Ok(RingPoint {
                position,
                node: NodeId(id),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(placement_machines::ring_owner(key.0, &ring).map(|node| node.0))
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("PlacementMachines")?;
    module.const_set("REPLICA_COUNT", REPLICA_COUNT)?;
    module.const_set("MAX_NODES", MAX_NODES)?;
    module.const_set("MAX_RING_POINTS", MAX_RING_POINTS)?;
    module.define_singleton_method("score", function!(score, 2))?;
    module.define_singleton_method("replicas", function!(replicas, 2))?;
    module.define_singleton_method("rendezvous_owner", function!(rendezvous_owner, 2))?;
    module.define_singleton_method("ring_owner", function!(ring_owner, 2))
}
