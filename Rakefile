# frozen_string_literal: true

require 'rake/testtask'
require 'rb_sys/extensiontask'
require 'rubygems/package'

GEMSPEC = Gem::Specification.load('placement_machines.gemspec')

RbSys::ExtensionTask.new('placement_machines_native', GEMSPEC) do |extension|
  extension.lib_dir = 'lib/placement_machines_native'
  extension.tmp_dir = 'tmp/rb_sys'
end

Rake::TestTask.new(:ruby_test) do |task|
  task.libs << 'lib'
  task.libs << 'test'
  task.pattern = 'test/**/*_test.rb'
end

desc 'Compile the native extension and test Ruby boundaries'
task test: %i[compile ruby_test]

desc 'Validate Ruby type signatures'
task :rbs do
  sh 'rbs', '-I', 'sig', 'validate'
end

desc 'Build a local source gem (no release or publication tasks)'
task :build do
  mkdir_p 'pkg'
  Gem::Package.build(GEMSPEC, false, false, "pkg/placement_machines-#{GEMSPEC.version}.gem")
end

desc 'Test the standalone Rust core'
task :cargo do
  sh 'cargo', 'test'
end

task default: %i[test cargo rbs]

Dir[File.join(__dir__, 'tasks/*.rake')].sort.each { |f| load f }
