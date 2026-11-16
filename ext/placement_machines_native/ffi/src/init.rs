use magnus::{Error, Ruby, function, prelude::*};
use matryona_machines::ruby::Class;

use crate::{CLASSES, Classes, node, replicas, ring_owner, ring_point, score};

/// Defines the placement functions and resolves the classes the gem defined.
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
