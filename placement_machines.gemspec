# frozen_string_literal: true

require_relative 'lib/placement_machines/version'

Gem::Specification.new do |spec|
  spec.name = 'placement_machines'
  spec.version = PlacementMachines::VERSION
  spec.authors = ['Abdelkader Boudih']
  spec.email = ['oss@seuros.com']
  spec.summary = 'Native rendezvous and ring placement decisions for Ruby'
  spec.description = 'A thin binding to the no_std placement-machines Rust core. ' \
                     'Provides deterministic rendezvous scores, ring ownership and ' \
                     'distinct-domain replica choices without membership services.'
  spec.homepage = 'https://github.com/seuros/placement_machines'
  spec.license = 'MIT'
  spec.required_ruby_version = '>= 4.0.0'
  spec.metadata['source_code_uri'] = spec.homepage
  spec.metadata['bug_tracker_uri'] = "#{spec.homepage}/issues"
  spec.metadata['rubygems_mfa_required'] = 'true'
  spec.metadata['cargo_crate_name'] = 'placement_machines_native'
  spec.metadata['cargo_manifest_path'] = 'ext/placement_machines_native/ffi/Cargo.toml'

  # Rust includes the core README at compile time.
  spec.files = Dir.glob(%w[
    lib/**/*.rb sig/**/*.rbs ext/**/*.{rb,rs,toml,md} ext/**/LICENSE
    Cargo.toml Cargo.lock README.md CHANGELOG.md LICENSE.txt
  ]).select { |path| File.file?(path) }
  spec.require_paths = ['lib']
  spec.extensions = ['ext/placement_machines_native/extconf.rb']
  spec.add_dependency 'rb_sys', '~> 0.9', '>= 0.9.130'
end
