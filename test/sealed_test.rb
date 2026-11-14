# frozen_string_literal: true

require 'minitest/autorun'
require 'placement_machines'

# Node and RingPoint are sealed, and sealing stops accidents, not reflection:
# the binding checks the exact class of every value it reads and re-validates
# its members through the core's constructor.
class SealedTest < Minitest::Test
  P = PlacementMachines

  # A named subclass raises, but Ruby binds the constant before `inherited`
  # runs, so `ForgedNode` leaks; reopened, it skips Node's validation.
  begin
    class ForgedNode < PlacementMachines::Node; end
  rescue TypeError => e
    NAMED_SUBCLASS_ERROR = e
  end

  class ForgedNode
    def initialize(id:, domain:) = Data.instance_method(:initialize).bind_call(self, id:, domain:)
  end

  def test_values_are_sealed
    [P::Node, P::RingPoint].each do |value|
      assert_raises(NoMethodError) { value.allocate }
      assert_raises(TypeError) { Class.new(value) }
    end
    assert_kind_of TypeError, NAMED_SUBCLASS_ERROR
    assert_equal P::Node, ForgedNode.superclass, 'the constant leaks'
  end

  def test_a_forged_node_never_reaches_the_core
    nodes = [P::Node.new(1, 1)]
    [ForgedNode.new(2, 2), ForgedNode.new(-2, 'x')].each do |forged|
      error = assert_raises(TypeError) { P.replicas(7, nodes + [forged], 2) }
      assert_equal 'entry must be a PlacementMachines::Node', error.message
    end
    # A repeated id would earn :repeated_node; a forged one raises first.
    assert_raises(TypeError) { P.replicas(7, nodes + [ForgedNode.new(1, 1)], 2) }
    assert_raises(TypeError) { P.ring_owner(7, [ForgedNode.new(1, 1)]) }
  end

  def test_marshalled_values_are_revalidated
    node = Marshal.load(Marshal.dump(P::Node.new(1, 2)).gsub("i\x06".b, "i\xFA".b))
    assert_instance_of P::Node, node
    assert_equal(-1, node.id)
    assert_raises(RangeError) { P.replicas(7, [node], 1) }
    point = Marshal.load(Marshal.dump(P::RingPoint.new(1, 2)).gsub("i\x06".b, "i\xFA".b))
    assert_raises(RangeError) { P.ring_owner(7, [point]) }
    assert_equal P::Node.new(1, 2), Marshal.load(Marshal.dump(P::Node.new(1, 2)))
  end

  def test_a_value_with_a_singleton_class_is_still_that_value
    node = P::Node.new(3, 4)
    node.singleton_class
    assert_equal [P::Node.new(3, 4)], P.replicas(7, [node], 1)
  end
end
