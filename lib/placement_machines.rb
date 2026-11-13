# frozen_string_literal: true

require_relative 'placement_machines/version'

module PlacementMachines
  # The core's `Node`: a node id (the core's `NodeId`, a caller-chosen Integer)
  # and the failure domain it shares with every node carrying the same value.
  Node = Data.define(:id, :domain)

  # The core's `RingPoint`: a ring coordinate and the id of the node that owns
  # it (the core's `NodeId`).
  RingPoint = Data.define(:position, :node)
end

# There is no pure-Ruby policy fallback when the native extension is missing.
# It builds Node values, so the Data classes above must exist first.
require 'placement_machines_native/placement_machines_native'
