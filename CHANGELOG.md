# Changelog

## [0.0.3](https://github.com/MiyakoMeow/async-sevenz-rs/compare/v0.0.2...v0.0.3) (2025-11-23)


### Bug Fixes

* **manifest:** del futures ([e8e4602](https://github.com/MiyakoMeow/async-sevenz-rs/commit/e8e4602c7b9589b8707af4574d5281a130dc307a))
* **test:** switch to tokio ([ff7d908](https://github.com/MiyakoMeow/async-sevenz-rs/commit/ff7d9087951878bf906110d3c6683b2d3f0d2350))
* **test:** use tokio::main in doc example ([d9528e8](https://github.com/MiyakoMeow/async-sevenz-rs/commit/d9528e8aa210e6fe1ce0a43e135f46db9d98f502))
* use ([8ae9497](https://github.com/MiyakoMeow/async-sevenz-rs/commit/8ae94978e983708f0f8dd7bf10c3839b379a5772))

## 0.0.2 (2025-11-22)


### ⚠ BREAKING CHANGES

* Rename identifiers to follow Rust API Guidelines

### Features

* Add `AutoFinisher` for `ArchiveWriter` ([5caaece](https://github.com/MiyakoMeow/async-sevenz-rs/commit/5caaece41917d8bcf36bffb621c8f0fc8c2be41b))
* add aes256sha256 method ([d0a09d8](https://github.com/MiyakoMeow/async-sevenz-rs/commit/d0a09d8e8d53ff00160cd74417f04cdf25663664))
* **ci:** move to renovate + release-please + publish ([df1c489](https://github.com/MiyakoMeow/async-sevenz-rs/commit/df1c489dc5614bd9d59846a31e7a11d431be7dba))
* **reader:** 添加异步打开7z存档文件的功能 ([82c56ac](https://github.com/MiyakoMeow/async-sevenz-rs/commit/82c56acbef62a1af5898ece7a400560a90583df0))
* solid compression ([5075e74](https://github.com/MiyakoMeow/async-sevenz-rs/commit/5075e74f3f5a090539973bbe9e72f133d34a6daa))
* support encrypted compression ([5bd2c84](https://github.com/MiyakoMeow/async-sevenz-rs/commit/5bd2c84c9466175d42942a4507e2fffffee3db22))
* **压缩:** 将LZMA实现替换为async-compression的LZMA2 ([30d47a0](https://github.com/MiyakoMeow/async-sevenz-rs/commit/30d47a0a85fe3bcc1fd0852004a0944026cae529))
* 添加异步文件解压缩支持 ([c74d59a](https://github.com/MiyakoMeow/async-sevenz-rs/commit/c74d59abee033184fd5ea182d1ac2e02d7982217))


### Bug Fixes

* **ci/rust:** use cache in all steps ([da11c2a](https://github.com/MiyakoMeow/async-sevenz-rs/commit/da11c2a96a1d702055b1c7d9c74b9026fdb8accf))
* **ci:** disable wasm check ([1106856](https://github.com/MiyakoMeow/async-sevenz-rs/commit/1106856b2a1c55e548af496c36d90431ec3c6aba))
* clippy ([ee8e17f](https://github.com/MiyakoMeow/async-sevenz-rs/commit/ee8e17fb13f7d7149a6b5b4f77b82cf974aa03ea))
* **docs:** fix README deps ([4e9abde](https://github.com/MiyakoMeow/async-sevenz-rs/commit/4e9abde0f694d9fab4329e5cfbae52d5bb63047b))
* extract empty file ([25ac087](https://github.com/MiyakoMeow/async-sevenz-rs/commit/25ac0870f21f094fda0e276a4a09079add7f2501))
* Incorrect handling of 7z time ([0628587](https://github.com/MiyakoMeow/async-sevenz-rs/commit/0628587c2b506ccaa4126c0fb4ba66a7c87d2d10))
* 移除重复的cfg属性和改进代码格式 ([a9e0e83](https://github.com/MiyakoMeow/async-sevenz-rs/commit/a9e0e83914ff2688e5f1a7359231727c2809757f))


### Styles

* Rename identifiers to follow Rust API Guidelines ([f47ddb8](https://github.com/MiyakoMeow/async-sevenz-rs/commit/f47ddb895d43f5784acc54506efa1801120c9312))


### Miscellaneous Chores

* change release ([f64ca57](https://github.com/MiyakoMeow/async-sevenz-rs/commit/f64ca57e994e56b77956c1209eea56fb0df92a40))
