# Changelog

## [0.4.0](https://github.com/seuros/placement_machines/compare/placement-machines-v0.3.0...placement-machines-v0.4.0) (2026-11-14)


### ⚠ BREAKING CHANGES

* validate Node and RingPoint through the core, seal values, document replicas cost

### Features

* validate Node and RingPoint through the core, seal values, document replicas cost ([c1e59de](https://github.com/seuros/placement_machines/commit/c1e59de9b6f471287d8184dae5367651aefe60ad))

## [0.3.0](https://github.com/seuros/placement_machines/compare/placement-machines-v0.2.2...placement-machines-v0.3.0) (2026-11-13)


### ⚠ BREAKING CHANGES

* converted to the fleet conventions. One exhaustive Refusal per crate, Result from every operation that can refuse, a refusal changes nothing, provable idempotent repeats are honoured, Ruby mirrors the core. No compatibility aliases. replicas(key, nodes, out) returns Result<usize, Refusal>; Ruby takes Node and RingPoint values.

### Features

* write replicas to an output buffer and refuse repeated node ids ([420b38e](https://github.com/seuros/placement_machines/commit/420b38eb0131db322e38d82728a84e0369ab98bf))

## [0.2.2](https://github.com/seuros/placement_machines/compare/placement-machines-v0.2.1...placement-machines-v0.2.2) (2026-11-06)


### Bug Fixes

* crates.io keywords and categories for the core ([997f0d7](https://github.com/seuros/placement_machines/commit/997f0d70e6292afdf3212730e3a1ac2e4662b86e))

## [0.2.1](https://github.com/seuros/placement_machines/compare/placement-machines-v0.2.0...placement-machines-v0.2.1) (2026-11-06)


### Bug Fixes

* keep test sources out of the packaged gem and crate ([18a75e8](https://github.com/seuros/placement_machines/commit/18a75e8cdf890fcff09732c53191b957db3fd43d))

## [0.2.0](https://github.com/seuros/placement_machines/compare/placement-machines-v0.1.0...placement-machines-v0.2.0) (2026-11-06)


### Features

* initial extraction ([eb26a4d](https://github.com/seuros/placement_machines/commit/eb26a4d14c4ee79cbd0f6afc8edbd8b55e598bae))
