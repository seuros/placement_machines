# frozen_string_literal: true

require_relative 'placement_machines/version'

module PlacementMachines
  # The core's `Node`: a node id (the core's `NodeId`, a caller-chosen Integer)
  # and the failure domain it shares with every node carrying the same value.
  # Built through the core's `Node::new`: both fields are Integers in
  # 0..2**64-1, anything else raises TypeError (not an Integer) or RangeError.
  Node = Data.define(:id, :domain) do
    def initialize(id:, domain:)
      id, domain = PlacementMachines.__send__(:__node, id, domain)
      super
    end
  end

  # The core's `RingPoint`: a ring coordinate and the id of the node that owns
  # it (the core's `NodeId`). Built through the core's `RingPoint::new`, under
  # the same rule as Node.
  RingPoint = Data.define(:position, :node) do
    def initialize(position:, node:)
      position, node = PlacementMachines.__send__(:__ring_point, position, node)
      super
    end
  end
end

# There is no pure-Ruby policy fallback when the native extension is missing.
# It validates and builds the Data values above.
require 'placement_machines_native/placement_machines_native'

module PlacementMachines
  private_class_method :__node, :__ring_point

  # The values are final. `initialize` is where each validates, and a subclass
  # could override it, or `allocate` build an instance that never ran it:
  # subclassing raises TypeError and `allocate` is gone. This stops accidents,
  # not reflection, so the binding also checks every value it reads.
  [Node, RingPoint].each do |value|
    value.singleton_class.undef_method(:allocate)
    value.define_singleton_method(:inherited) { |_| raise TypeError, "#{value} cannot be subclassed" }
  end
end
