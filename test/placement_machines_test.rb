# frozen_string_literal: true

require 'minitest/autorun'
require 'placement_machines'

class PlacementMachinesTest < Minitest::Test
  P = PlacementMachines
  MAX = (1 << 64) - 1
  NODES = [[1, 10], [2, 20], [3, 30]].map { |id, domain| P::Node.new(id, domain) }.freeze

  def test_native_api_and_empty_membership
    assert_nil P.method(:replicas).source_location
    assert_match(/\A\d+\.\d+\.\d+\z/, P::VERSION)
    assert_equal %i[id domain], P::Node.members
    assert_equal %i[position node], P::RingPoint.members
    assert_equal [], P.replicas(0, [], 3)
    assert_nil P.ring_owner(0, [])
    refute P.respond_to?(:rendezvous_owner)
    refute P.const_defined?(:REPLICA_COUNT)
  end

  def test_rendezvous_is_stable_and_removal_only_moves_owned_keys
    100.times do |key|
      owner = P.replicas(key, NODES, 1).first
      assert_instance_of P::Node, owner
      assert_equal owner, P.replicas(key, NODES.reverse, 1).first
      assert_equal owner, P.replicas(key, NODES, 3).first
      assert_equal owner, P.replicas(key, NODES.first(2), 1).first unless owner.id == 3
      best = NODES.max_by { |node| [P.score(key, node.id), -node.id] }
      assert_equal best, owner
    end
  end

  def test_replicas_use_distinct_domains_without_fabricating_members
    membership = [[1, 10], [2, 10], [3, 30]].map { |id, domain| P::Node.new(id, domain) }.freeze
    result = P.replicas(42, membership, 3)
    assert_equal 2, result.size
    assert_equal 2, result.map(&:domain).uniq.size
    assert(result.all? { |node| membership.include?(node) })
    assert_equal [P::Node.new(MAX, MAX)], P.replicas(MAX, [P::Node.new(MAX, MAX)], 3)
  end

  def test_limit_is_the_length_of_the_cores_output_buffer
    full = P.replicas(7, NODES, 3)
    assert_equal 3, full.size
    assert_equal [], P.replicas(7, NODES, 0)
    assert_equal full.first(1), P.replicas(7, NODES, 1)
    assert_equal full.first(2), P.replicas(7, NODES, 2)
    assert_equal full, P.replicas(7, NODES, 9)
    assert_equal full, P.replicas(7, NODES, MAX)
    assert_raises(ArgumentError) { P.replicas(7, NODES) }
    assert_raises(TypeError) { P.replicas(7, NODES, 1.0) }
    assert_raises(TypeError) { P.replicas(7, NODES, nil) }
    assert_raises(RangeError) { P.replicas(7, NODES, -1) }
  end

  def test_a_repeated_node_id_is_a_refusal_not_an_exception
    repeated = [P::Node.new(1, 1), P::Node.new(1, 2)].freeze
    assert_equal :repeated_node, P.replicas(1, repeated, 3)
    assert_equal :repeated_node, P.replicas(1, repeated, 0)
    assert_equal :repeated_node, P.replicas(1, NODES + [NODES.first], 1)
    assert_equal [P::Node.new(1, 1), P::Node.new(1, 2)], repeated
  end

  def test_ring_exact_match_wraparound_unsorted_and_ties
    ring = [[30, 1], [10, 2], [10, 0]].map { |position, node| P::RingPoint.new(position, node) }.freeze
    assert_equal 1, P.ring_owner(11, ring)
    assert_equal 0, P.ring_owner(31, ring)
    assert_equal 0, P.ring_owner(10, ring)
    assert_equal 0, P.ring_owner(MAX, ring)
    assert_equal MAX, P.ring_owner(MAX, [P::RingPoint.new(MAX, MAX)])
    assert_equal 1, P.ring_owner(11, [P::RingPoint.new(30, 1), P::RingPoint.new(10, 1)])
  end

  def test_strict_numeric_types_and_unsigned_bounds
    coercible = Object.new
    def coercible.to_int = 1
    [1.0, 1.5, Float::NAN, Float::INFINITY, nil, true, '1', coercible].each do |invalid|
      assert_raises(TypeError) { P.score(invalid, 0) }
      assert_raises(TypeError) { P.score(0, invalid) }
      assert_raises(TypeError) { P.replicas(invalid, [], 3) }
      assert_raises(TypeError) { P.replicas(0, [P::Node.new(invalid, 0)], 3) }
      assert_raises(TypeError) { P.replicas(0, [P::Node.new(0, invalid)], 3) }
      assert_raises(TypeError) { P.ring_owner(0, [P::RingPoint.new(invalid, 0)]) }
      assert_raises(TypeError) { P.ring_owner(0, [P::RingPoint.new(0, invalid)]) }
    end
    [-1, MAX + 1].each do |invalid|
      assert_raises(RangeError) { P.score(invalid, 0) }
      assert_raises(RangeError) { P.score(0, invalid) }
      assert_raises(RangeError) { P.replicas(0, [P::Node.new(0, invalid)], 3) }
      assert_raises(RangeError) { P.ring_owner(0, [P::RingPoint.new(0, invalid)]) }
    end
    assert_kind_of Integer, P.score(MAX, MAX)
    assert_operator P.score(MAX, MAX), :<=, MAX
  end

  def test_strict_shapes_and_explicit_collection_bounds
    coercible = Object.new
    def coercible.to_ary = [P::Node.new(1, 1)]
    [nil, {}, coercible].each do |invalid|
      assert_raises(TypeError) { P.replicas(0, invalid, 3) }
      assert_raises(TypeError) { P.ring_owner(0, invalid) }
    end
    lookalike = Data.define(:id, :domain).new(1, 1)
    [nil, [1, 1], lookalike, P::RingPoint.new(1, 1)].each do |invalid|
      assert_raises(TypeError) { P.replicas(0, [invalid], 3) }
    end
    [nil, [1, 1], P::Node.new(1, 1)].each do |invalid|
      assert_raises(TypeError) { P.ring_owner(0, [invalid]) }
    end
    nodes = P::MAX_NODES.times.map { |id| P::Node.new(id, id) }
    assert_equal 3, P.replicas(0, nodes, 3).size
    assert_equal P::MAX_NODES, P.replicas(0, nodes, MAX).size
    assert_raises(RangeError) { P.replicas(0, nodes + [P::Node.new(1000, 1000)], 3) }
    points = P::MAX_RING_POINTS.times.map { |id| P::RingPoint.new(id, id) }
    assert_equal 0, P.ring_owner(0, points)
    assert_raises(RangeError) { P.ring_owner(0, points + [P::RingPoint.new(10_000, 10_000)]) }
  end

  def test_randomized_replicas_match_a_ranked_reference
    random = Random.new(4213)
    300.times do
      nodes = Array.new(random.rand(0..8)) { P::Node.new(random.rand(0..20), random.rand(0..3)) }
      key = random.rand(0..MAX)
      limit = random.rand(0..5)
      answer = P.replicas(key, nodes, limit)
      if nodes.map(&:id).uniq.size < nodes.size
        assert_equal :repeated_node, answer
      else
        ranked = nodes.sort_by { |node| [-P.score(key, node.id), node.id] }
        expected = ranked.uniq(&:domain).first(limit)
        assert_equal expected, answer
      end
      assert_equal answer, P.replicas(key, nodes, limit)
    end
  end

  def test_frozen_inputs_and_gc_are_safe_without_retaining_ruby_values
    expected = P.replicas(42, NODES, 3)
    100.times do
      assert_equal expected, P.replicas(42, NODES, 3)
      P.ring_owner(MAX, [P::RingPoint.new(0, MAX)].freeze)
    end
    GC.start
    GC.compact if GC.respond_to?(:compact)
    assert_equal expected, P.replicas(42, NODES, 3)
    expected.clear
    assert_equal 3, P.replicas(42, NODES, 3).size
  end
end
