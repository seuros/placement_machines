# frozen_string_literal: true

require 'minitest/autorun'
require 'placement_machines'

class PlacementMachinesTest < Minitest::Test
  P = PlacementMachines
  MAX = (1 << 64) - 1
  NODES = [[1, 10], [2, 20], [3, 30]].map(&:freeze).freeze

  def test_native_api_and_empty_membership
    assert_nil P.method(:replicas).source_location
    assert_match(/\A\d+\.\d+\.\d+\z/, P::VERSION)
    assert_equal 3, P::REPLICA_COUNT
    assert_equal [nil, nil, nil], P.replicas(0, [])
    assert_nil P.rendezvous_owner(0, [])
    assert_nil P.ring_owner(0, [])
  end

  def test_rendezvous_is_stable_and_removal_only_moves_owned_keys
    100.times do |key|
      owner = P.rendezvous_owner(key, NODES)
      assert_equal owner, P.rendezvous_owner(key, NODES.reverse)
      assert_equal owner, P.replicas(key, NODES).first
      assert_equal owner, P.rendezvous_owner(key, NODES.first(2)) unless owner == 3
    end
  end

  def test_replicas_use_distinct_domains_without_fabricating_members
    membership = [[1, 10], [2, 10], [3, 30]].map(&:freeze).freeze
    result = P.replicas(42, membership)
    assert_equal 3, result.size
    assert_nil result.last
    assert_equal 2, result.compact.size
    domains = membership.to_h
    assert_equal 2, result.compact.map { |id| domains.fetch(id) }.uniq.size
    assert_equal [MAX, nil, nil], P.replicas(MAX, [[MAX, MAX]])
    assert_raises(ArgumentError) { P.replicas(1, [[1, 1], [1, 2]]) }
    assert_raises(ArgumentError) { P.rendezvous_owner(1, [[1, 1], [1, 2]]) }
  end

  def test_ring_exact_match_wraparound_unsorted_and_ties
    ring = [[30, 1], [10, 2], [10, 0]].map(&:freeze).freeze
    assert_equal 1, P.ring_owner(11, ring)
    assert_equal 0, P.ring_owner(31, ring)
    assert_equal 0, P.ring_owner(10, ring)
    assert_equal 0, P.ring_owner(MAX, ring)
    assert_equal MAX, P.ring_owner(MAX, [[MAX, MAX]])
    assert_equal 1, P.ring_owner(11, [[30, 1], [10, 1]])
  end

  def test_strict_numeric_types_and_unsigned_bounds
    coercible = Object.new
    def coercible.to_int = 1
    [1.0, 1.5, Float::NAN, Float::INFINITY, nil, true, '1', coercible].each do |invalid|
      assert_raises(TypeError) { P.score(invalid, 0) }
      assert_raises(TypeError) { P.score(0, invalid) }
      assert_raises(TypeError) { P.replicas(0, [[invalid, 0]]) }
      assert_raises(TypeError) { P.replicas(0, [[0, invalid]]) }
      assert_raises(TypeError) { P.ring_owner(0, [[invalid, 0]]) }
      assert_raises(TypeError) { P.ring_owner(0, [[0, invalid]]) }
    end
    [-1, MAX + 1].each do |invalid|
      assert_raises(RangeError) { P.score(invalid, 0) }
      assert_raises(RangeError) { P.score(0, invalid) }
      assert_raises(RangeError) { P.replicas(0, [[0, invalid]]) }
      assert_raises(RangeError) { P.ring_owner(0, [[0, invalid]]) }
    end
    assert_kind_of Integer, P.score(MAX, MAX)
    assert_operator P.score(MAX, MAX), :<=, MAX
  end

  def test_strict_array_shapes_and_explicit_collection_bounds
    coercible = Object.new
    def coercible.to_ary = [[1, 1]]
    [nil, {}, coercible].each do |invalid|
      assert_raises(TypeError) { P.replicas(0, invalid) }
      assert_raises(TypeError) { P.ring_owner(0, invalid) }
    end
    [[], [1], [1, 2, 3]].each do |invalid|
      assert_raises(ArgumentError) { P.replicas(0, [invalid]) }
      assert_raises(ArgumentError) { P.ring_owner(0, [invalid]) }
    end
    assert_raises(TypeError) { P.replicas(0, [nil]) }
    nodes = P::MAX_NODES.times.map { |id| [id, id] }
    assert_equal 3, P.replicas(0, nodes).compact.size
    assert_raises(ArgumentError) { P.replicas(0, nodes + [[1000, 1000]]) }
    points = P::MAX_RING_POINTS.times.map { |id| [id, id] }
    assert_equal 0, P.ring_owner(0, points)
    assert_raises(ArgumentError) { P.ring_owner(0, points + [[10000, 10000]]) }
  end

  def test_frozen_inputs_and_gc_are_safe_without_retaining_ruby_values
    expected = P.replicas(42, NODES)
    assert_equal [[1, 10], [2, 20], [3, 30]], NODES
    100.times do
      assert_equal expected, P.replicas(42, NODES)
      P.ring_owner(MAX, [[0, MAX]].freeze)
    end
    GC.start
    GC.compact if GC.respond_to?(:compact)
    assert_equal expected, P.replicas(42, NODES)
    expected.clear
    assert_equal 3, P.replicas(42, NODES).size
  end
end
