# frozen_string_literal: true

require_relative 'placement_machines/version'

# There is no pure-Ruby policy fallback when the native extension is missing.
require 'placement_machines_native/placement_machines_native'
