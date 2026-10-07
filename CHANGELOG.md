# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2026-09-27

### Added

- *(iso-prototype)* Enhance reference-head with facial hair, glasses, expressions([c68bfc6](https://github.com/scbrown/pixelsrc/commit/c68bfc681c5b19b294cb22c801bcbd7d489ef3af))
- Rebuild examples/demos directory with regions format([ea266fe](https://github.com/scbrown/pixelsrc/commit/ea266fe9f55f84f2f99e2b742f1722735fcde9c9))
- *(demos)* Add demos for transform, agent-verify, and agent commands([fc06cbb](https://github.com/scbrown/pixelsrc/commit/fc06cbbf8b48e3bb3a83b408066513310a96d8e5))
- *(examples)* Add eyebrow expression renders for reference-head([3f32ee1](https://github.com/scbrown/pixelsrc/commit/3f32ee177b76d336e5ecbb85d6242d1e535c2777))
- *(examples)* Add avatar-prototype with skull sprite via artisan workflow([6c33833](https://github.com/scbrown/pixelsrc/commit/6c338335599978f4dae21645bbfe5bcf724428fa))
- *(examples)* Add talking skull animation with mouth states([f6c75ba](https://github.com/scbrown/pixelsrc/commit/f6c75bacc33ef56215555a4a75e6a7b668a3b7e2))
- *(cli)* Remove deprecated pxl transform command([6e0ebaf](https://github.com/scbrown/pixelsrc/commit/6e0ebaf18fc592f59a97b5c7a2370a9c15b6d9cf))
- *(examples)* Add skull-outline silhouette study([08dcc51](https://github.com/scbrown/pixelsrc/commit/08dcc51768f6993b3a78372420e7bd59c29ed3c6))
- *(demos)* Add animation demo tests (DT-4)([9dcfb36](https://github.com/scbrown/pixelsrc/commit/9dcfb36a13b23d54667a5e0c134ab57b22d68e52))
- *(demos)* Add composition demo tests (DT-5)([f4658ef](https://github.com/scbrown/pixelsrc/commit/f4658efd557880a5c5d5702936e14f14de1d9f84))
- *(demos)* Add @demo annotations to export tests (DT-6/7/8)([d4325f3](https://github.com/scbrown/pixelsrc/commit/d4325f3ffe88d5898b11178964ba70c41704f6dd))
- *(demos)* Add CSS color demo tests (DT-9)([886b3e3](https://github.com/scbrown/pixelsrc/commit/886b3e3afda25e628c08875c09241d86cdf38fee))
- *(sprites)* Add beanie hat and burly sailor beard([b9f3825](https://github.com/scbrown/pixelsrc/commit/b9f382511358fbc9f8a5cc90a8181b1c9c7d929f))
- *(examples)* Add neutral eyebrow sprite and style variations([145584f](https://github.com/scbrown/pixelsrc/commit/145584fc45b0bc170bdbf7607b21d8f7081fa4d7))
- *(demos)* Add CSS timing function demo tests (DT-11)([fbbe283](https://github.com/scbrown/pixelsrc/commit/fbbe283fddd37c67f5ea13cfc28159a9c8a16bd7))
- *(demos)* Add CSS transform demo tests (DT-12)([91ab190](https://github.com/scbrown/pixelsrc/commit/91ab190401541151f9819a817272f9a9b5c13ac2))
- *(demos)* Add palette cycling demo tests (DT-11)([b9d1833](https://github.com/scbrown/pixelsrc/commit/b9d1833449f3c808d59a991d0c16a8e6295b3c97))
- *(demos)* Add CSS keyframes demo tests (DT-14)([47d25f0](https://github.com/scbrown/pixelsrc/commit/47d25f0a3ba3d5fafd873f523c1ff5aebe5d1cc6))
- *(demos)* Add @demo annotations to build system tests (DT-13)([2af6f3d](https://github.com/scbrown/pixelsrc/commit/2af6f3de3652cd62f454fc75a623655e62b43bd1))
- *(docs)* Add CSS section to mdbook with demo markers (DT-17)([bb9199f](https://github.com/scbrown/pixelsrc/commit/bb9199f9cb8a395e6f2ba4dd8716424496e91b57))
- *(demos)* Add @demo annotations to CLI tests([63a30a5](https://github.com/scbrown/pixelsrc/commit/63a30a595037c7e0abf66baa76eac8765058681b))
- *(ci)* Enforce demo coverage threshold([8ba6cab](https://github.com/scbrown/pixelsrc/commit/8ba6cab5b06428003389ef2083a5f9624ad3c2ed))
- *(ci)* Add demo tests and documentation checks to CI (DT-18)([5bc2e01](https://github.com/scbrown/pixelsrc/commit/5bc2e01ef98c0d49cb00cf1b485ee6363609d480))
- *(artisan)* Skull sprite iteration round 2([8fca6be](https://github.com/scbrown/pixelsrc/commit/8fca6be10827e5aa9ac4cdcf64a5fff2652a5373))
- *(antialias)* Add core types and enums([c5cd7e3](https://github.com/scbrown/pixelsrc/commit/c5cd7e39ba4bd78223ff89a511a7b874aec36ab1))
- *(config)* Add AntialiasConfig to schema([bd8fb98](https://github.com/scbrown/pixelsrc/commit/bd8fb98bb8e649da7e0828a62cacff2f5e78cd09))
- *(cli)* Add antialiasing CLI flags to render command([75bc99e](https://github.com/scbrown/pixelsrc/commit/75bc99e302d53e6d14563ac553c94327d7658965))
- *(antialias)* Add SemanticContext for AA extraction([9111765](https://github.com/scbrown/pixelsrc/commit/9111765a5658879208449791858e6030e4a2ba4f))
- *(antialias)* Add apply_antialias() pipeline integration([1fe1679](https://github.com/scbrown/pixelsrc/commit/1fe167904418b1d1ba09ccc2e0d717a8374125bb))
- *(antialias)* Add aa-blur algorithm with semantic masking([3e73a93](https://github.com/scbrown/pixelsrc/commit/3e73a93aa571fedecdb527c7aa9d87eb44b64142))
- *(antialias)* Add Scale2x algorithm with semantic modifications([d040d38](https://github.com/scbrown/pixelsrc/commit/d040d3805c261c940746da3f6d5d9cbf7efec7b0))
- *(antialias)* Add HQ2x/HQ4x algorithms with semantic awareness([45b349d](https://github.com/scbrown/pixelsrc/commit/45b349de6f92f88922c30d98f3fa1f8fe6fdb620))
- *(antialias)* Implement xBR2x/xBR4x edge-aware upscaling([46ec422](https://github.com/scbrown/pixelsrc/commit/46ec42251f64a5b7043d435dd02586efc025caa2))
- *(antialias)* Implement gradient smoothing for DerivesFrom relationships([052358f](https://github.com/scbrown/pixelsrc/commit/052358f13f705f777c2105d57edd6bd371fd3e20))
- *(antialias)* Add @demo annotated tests for AA algorithms and semantic preservation([6daa4e2](https://github.com/scbrown/pixelsrc/commit/6daa4e2cc789ac57ef0ca002aa8a3f3ad57eb78f))
- *(models)* Add per-sprite and per-region antialias config([016b983](https://github.com/scbrown/pixelsrc/commit/016b983160d8ddc80c408d70e88a320f6fc69d74))
- *(demo-tests)* Add @demo annotations for 100% coverage([93f12a0](https://github.com/scbrown/pixelsrc/commit/93f12a0008d1d75d9df99357bcbe5f08ece4238d))
- *(antialias)* Add comprehensive integration tests([f9e45c7](https://github.com/scbrown/pixelsrc/commit/f9e45c7398976792f169a78eee72242edeac8071))
- *(idioms)* Add #[non_exhaustive] to all public error enums([bc16a5e](https://github.com/scbrown/pixelsrc/commit/bc16a5e1482657b3963ca6e8227c30266e78c0be))
- *(demo-tests)* Add CSS variable demo tests (DT-10)([658a6b2](https://github.com/scbrown/pixelsrc/commit/658a6b23c79176ba0c2f7e0adb8175681421d539))
- *(ergonomics)* Add Result type aliases to major modules([e0933ea](https://github.com/scbrown/pixelsrc/commit/e0933ea13272e1c4968a0bf0d48a7fd6f89c1426))
- *(antialias)* Add example sprites for AA demo([f650089](https://github.com/scbrown/pixelsrc/commit/f6500893f8c7bdd92baf475af4f34dced0d45ba6))
- *(demos)* Add @demo annotations to CLI tests([c7f201a](https://github.com/scbrown/pixelsrc/commit/c7f201a93c600971877bb626dacd36a7a43363bd))
- Establish 3/4 view perspective for skull sprite([be1b2f6](https://github.com/scbrown/pixelsrc/commit/be1b2f6e960a0512ff096cd823766a63d9f03eb0))
- Add pyo3 dependency and python feature flag([8b5c254](https://github.com/scbrown/pixelsrc/commit/8b5c254887a8d9c14d589b5e7ce2781f2f260f8c))
- Scaffold Python package with PyO3 + maturin (PY-1, PY-2)([1f477b9](https://github.com/scbrown/pixelsrc/commit/1f477b948551c3f377b11ec2e555a7fcac7b0366))
- Add parse, list_sprites, list_palettes Python bindings (PY-4)([a33cf45](https://github.com/scbrown/pixelsrc/commit/a33cf45fbaca32ec07ee9ca298db7a3432b9a840))
- Add validate, validate_file Python bindings (PY-5)([18d39ee](https://github.com/scbrown/pixelsrc/commit/18d39ee40cb829b84730d6e74330290257565ad8))
- Add import_png, import_png_analyzed Python bindings (PY-7)([204c707](https://github.com/scbrown/pixelsrc/commit/204c70745013e5e032857dee6e394f58818d63b1))
- Add format_pxl Python binding (PY-9)([2b6e777](https://github.com/scbrown/pixelsrc/commit/2b6e777855de6a633f8f3f06f01806c19244d5c2))
- Add stateful Registry #[pyclass] for Python bindings (PY-8)([a406799](https://github.com/scbrown/pixelsrc/commit/a406799a46dca215151ba55a5d39a978813a21ae))
- Add comprehensive Python test suite for color and import bindings (PY-11)([6e67037](https://github.com/scbrown/pixelsrc/commit/6e670372d76575376987bc1730d88e8b3013df82))
- Add missing demo test annotations to reach 100% coverage([02a54c9](https://github.com/scbrown/pixelsrc/commit/02a54c946b45f580c2f44111780b5f8f67918253))
- Add banner sprite definitions (examples/banner_pxl.jsonl)([13b5dc4](https://github.com/scbrown/pixelsrc/commit/13b5dc46972d601819a574998d7af71592ab98b6))
- Generate favicon at multiple sizes + ICO + manifest.json (Phase 13.4)([61d3c12](https://github.com/scbrown/pixelsrc/commit/61d3c12be2e369de22c67106ca54b0b6c9664020))
- Create social preview OG image (1280x640) (Phase 13.5)([daaae27](https://github.com/scbrown/pixelsrc/commit/daaae27c9a444a8e60f770b8dbfb0403de91adfb))
- Add pxl draw subcommand with file read-modify-write pipeline([12582fc](https://github.com/scbrown/pixelsrc/commit/12582fcbdf63b53a54b06849edd140db8d72d07e))
- Add grid token editing engine with --set and --erase operations([37968bc](https://github.com/scbrown/pixelsrc/commit/37968bc108a7f2c75206754f7ee3d1bf1489281c))
- Add --line x1,y1,x2,y2={token} draw operation using Bresenham algorithm([94e2edc](https://github.com/scbrown/pixelsrc/commit/94e2edc6286973a050599aa4ca8136fbc01cd355))
- Add `pxl scaffold` CLI subcommand with sprite/composition/palette dispatch([1b884b5](https://github.com/scbrown/pixelsrc/commit/1b884b5ca94a106b3b4eeeb28ca7e61a83066674))
- Add pxl mask subcommand with token grid extraction([05f9074](https://github.com/scbrown/pixelsrc/commit/05f90742b2983148ac6d4a70f9d640c6e5a4572b))
- Add MCP tools for format, prime, palettes, analyze, scaffold([6e4c3f0](https://github.com/scbrown/pixelsrc/commit/6e4c3f0b719b0cf11c58180c9d67b8f13f795a9e))
- Implement pixelsrc_import MCP tool for PNG to .pxl conversion([ec09a3e](https://github.com/scbrown/pixelsrc/commit/ec09a3e790f3051427e90ae0d4922a06a0c0a177))
- Add pre-commit hook installer for fmt + clippy checks([45b143c](https://github.com/scbrown/pixelsrc/commit/45b143c1dc8c42567c4b005072f51063c6130197))
- Add MCP static resources for format spec and palette catalog([a9a5ecf](https://github.com/scbrown/pixelsrc/commit/a9a5ecff4cf0e890dbed90cf7f225ae2e9c01c42))
- Version-control pre-commit hooks via .githooks/ and core.hooksPath([3b23776](https://github.com/scbrown/pixelsrc/commit/3b23776c2f074813f6c6c4dc105826b5742e0c49))
- Add --query and --bounds operations to pxl mask([c021551](https://github.com/scbrown/pixelsrc/commit/c0215514f0b94d6431125db1abb5299835104a14))
- Implement --sample and --neighbors mask operations([36a2814](https://github.com/scbrown/pixelsrc/commit/36a281467ff0cc7817e1cf860e6459b9a58310cf))
- Add --list operation to pxl mask([18a4bcd](https://github.com/scbrown/pixelsrc/commit/18a4bcd6f654fdb0c6e49420c73d8693a4b9c8cf))
- Add --region and --count operations to pxl mask([bbcfb19](https://github.com/scbrown/pixelsrc/commit/bbcfb19f89e5831e21b3465d0031b9180ddad875))
- Add MCP integration tests + Claude Code/Desktop config examples([120825b](https://github.com/scbrown/pixelsrc/commit/120825bdbcc58aae6b8a626f2c241495d9f5aac6))
- Add MCP resource templates for palettes, examples, and prompts([1c4b9c0](https://github.com/scbrown/pixelsrc/commit/1c4b9c0043119b66b5125c9260cd4141b7c8d06c))
- Add mask CLI integration tests, agent workflow example, and docs([f7e7d3a](https://github.com/scbrown/pixelsrc/commit/f7e7d3a09060a3e2f9ab941df36f144ef4e64432))
- Add MCP prompt templates for guided pixelsrc workflows([d54824f](https://github.com/scbrown/pixelsrc/commit/d54824fc272337f3072b427b88d45806307cb539))
- Add draw tests, edge cases, and docs([132a81a](https://github.com/scbrown/pixelsrc/commit/132a81a23d6ae232858a075af0cbd4a53c38da95))
- Add pixelsrc_render MCP tool for .pxl to PNG rendering([1c7a06a](https://github.com/scbrown/pixelsrc/commit/1c7a06a167a02af153e684dc68f8ec92bbe4766f))
- Add pixelsrc_explain and pixelsrc_diff MCP tools([49e10cc](https://github.com/scbrown/pixelsrc/commit/49e10cc4e417897e13b3fb9292e05efd012e5485))
- Add pixelsrc_validate and pixelsrc_suggest MCP tools([0f7a823](https://github.com/scbrown/pixelsrc/commit/0f7a823b3c2ea1073d446d5a72ecb3eda0ade9d9))
- Add project-wide registry for cross-file item resolution([cb9c6a8](https://github.com/scbrown/pixelsrc/commit/cb9c6a8492991d059fc8d880f1b1398bc5377405))
- Add @include:path#name syntax for named palette selection([79bd1b1](https://github.com/scbrown/pixelsrc/commit/79bd1b1d9f35faa0f1d5d3a43d65fb29d134e401))
- Add Import object type with parser, validation, and explain support([657b178](https://github.com/scbrown/pixelsrc/commit/657b1782e764387d64e8ef749c3ef71e34c8260f))
- Implement two-pass build pipeline for cross-file reference resolution([90a16be](https://github.com/scbrown/pixelsrc/commit/90a16be59f296f6a1300b7e2ec9188b55f8a9949))
- Add project context auto-detection to pxl render([2234627](https://github.com/scbrown/pixelsrc/commit/22346277a32e2a5791d1810d38393e9049720b03))
- Add import resolution for cross-file references([57e1a20](https://github.com/scbrown/pixelsrc/commit/57e1a207b55e5cbb6ca374d1bf3367996d77d014))
- Add optional NumPy/Pillow integration for RenderResult([b53182d](https://github.com/scbrown/pixelsrc/commit/b53182d077666e0727c94f6405608614159984c5))
- Add VS Code extension for pixelsrc([2b3f62c](https://github.com/scbrown/pixelsrc/commit/2b3f62c59dc1cfe35811a35dcbd95ea2fb81c42d))
- Add particle system rendering engine([07d68ed](https://github.com/scbrown/pixelsrc/commit/07d68edc40a5e49e3a5e1c4f240eb8cc589aad91))
- Add circle and ellipse fill to pxl draw([348175f](https://github.com/scbrown/pixelsrc/commit/348175fb2d1a29318e39e12e32ebf48d3c92bbfc))
- Add MCP integration tests for explain/diff tools + docs([95ba850](https://github.com/scbrown/pixelsrc/commit/95ba85032b7603a2e923f7ddb583a08a53d1209b))
- Add [dependencies] section to pxl.toml schema([9cc0be1](https://github.com/scbrown/pixelsrc/commit/9cc0be12941abcc644a14449b6ed9de89fd3c532))
- Add LSP cross-file support via ProjectRegistry([d92ee71](https://github.com/scbrown/pixelsrc/commit/d92ee719e36d30722698ab80dfcfbd6c24e76dea))
- Add import validation and suggestions for pxl validate/suggest([00e8784](https://github.com/scbrown/pixelsrc/commit/00e8784f40b4c858d7ea52ed03f90814846ef514))
- Add pxl install command for dependency fetching([d14ffb1](https://github.com/scbrown/pixelsrc/commit/d14ffb1cb914d2aaaad92f2c3aad6984ca655716))
- Add namespaced external references via dependency loading([9f588ae](https://github.com/scbrown/pixelsrc/commit/9f588ae8a62b09639248a8dc8d6d7ea4d940dcf0))
- *(examples)* Skull sprite matching the reference([fd3f2d5](https://github.com/scbrown/pixelsrc/commit/fd3f2d55659419795a4d08aecdd60d4ac8ad9a4d))
- *(examples)* Animate the skull (idle, laugh, glowing eyes, speech timing)([3d9bfb3](https://github.com/scbrown/pixelsrc/commit/3d9bfb3bd7e7e48f2a19eb67ed4809d18cfb0a7c))
- Validate catches duplicate region keys, OOB pixels, empty regions, unused tokens([46a62e1](https://github.com/scbrown/pixelsrc/commit/46a62e1bf3a612debdd5095741deb4575e949f34))
- Diff compares region geometry and reports presence honestly([81794b1](https://github.com/scbrown/pixelsrc/commit/81794b1d84490a1e9834a6450df750405ece728c))
- Sprite `extends` inherits and patches regions at the key level([6cfce13](https://github.com/scbrown/pixelsrc/commit/6cfce13d36b92c4d0806214e1153daa27d0efcd0))
- Bake particle systems to frames with `pxl render --frames`([32dea13](https://github.com/scbrown/pixelsrc/commit/32dea138d07a228b0e3d77c67b9fdcdba2799c07))
- Per-region and sprite-level group `translate` (one-line walk bob)([dd8fff9](https://github.com/scbrown/pixelsrc/commit/dd8fff97fa0a4d7e1a4e2487a4267c8e66d22a31))
- Auto-outline renders a real silhouette outline([73e1d17](https://github.com/scbrown/pixelsrc/commit/73e1d17ebdc8fa30b2414638aad1a11ea01d83e2))
- `pxl show` renders structured sprites as an ANSI color preview([320a1f2](https://github.com/scbrown/pixelsrc/commit/320a1f2ce48bf061c00ab8fae7e088c9fa010ac7))

### CI/CD

- Add Python wheel build and PyPI publish workflow (PY-12)([bd08d45](https://github.com/scbrown/pixelsrc/commit/bd08d4507fe72dd74f6862e90dbd807633e2f43a))
- Release-plz with binary builds; fix red main([fc97085](https://github.com/scbrown/pixelsrc/commit/fc97085afa0d9d8c36fb835171fda515c6fde33d))
- Clippy late-init lint; drop native x86_64 linker override that broke wheel builds([a1342a0](https://github.com/scbrown/pixelsrc/commit/a1342a0d8acc5e5fe40fbad3e6e6757fd14e4774))
- Unbreak the Python wheel and browser-test lanes([4bb0f81](https://github.com/scbrown/pixelsrc/commit/4bb0f81909692511faaf4a0a07214e0a98fae8ce))
- Publish the Homebrew formula to the scbrown/homebrew-pixelsrc tap([8acb129](https://github.com/scbrown/pixelsrc/commit/8acb129ab00a5c9fdc61733354aac87e1ed071c1))
- Browser tests serve mdBook's html/ output; Linux wheels find interpreters([cc5f3a0](https://github.com/scbrown/pixelsrc/commit/cc5f3a085c417d79b39ec23fbcfb64302fe6d0d2))

### Changed

- *(fixtures)* Convert test fixtures from grid to regions format([a68e463](https://github.com/scbrown/pixelsrc/commit/a68e46391768dd830bded96f2b514e695edbffff))
- *(fixtures)* Convert remaining test fixtures from grid to regions([2cf5356](https://github.com/scbrown/pixelsrc/commit/2cf5356a0745253a543ddffc745df3f840e195a7))
- *(init)* Convert project templates from grid to regions format([524e7bb](https://github.com/scbrown/pixelsrc/commit/524e7bba1be50cdaa445c32a56512bd0cb8fd3ed))
- Remove grid format from scaffold templates and fmt output([d5ea8a1](https://github.com/scbrown/pixelsrc/commit/d5ea8a1289d4a8f4c852f938eacfd8845208ed2d))
- Convert test strings from grid to regions format([2a1e655](https://github.com/scbrown/pixelsrc/commit/2a1e655b17780d719944a577b6e85966ed034766))
- *(parser)* Convert test strings from grid to regions format([30a3d99](https://github.com/scbrown/pixelsrc/commit/30a3d990f5a8a3bd0ac0e497f535716224610c27))
- Remove grid format references from production and test code([93192fc](https://github.com/scbrown/pixelsrc/commit/93192fc38abac9a6e3873c0fee816188d72596d0))
- *(transforms)* Split transforms.rs into modular directory structure([ecdbc8d](https://github.com/scbrown/pixelsrc/commit/ecdbc8d0cd3045ded79a3a4fe6db006c82a3986e))
- Convert models.rs and registry.rs grid refs to regions([74b5351](https://github.com/scbrown/pixelsrc/commit/74b5351a9e516399c2d7293228e5e9ac6e473b26))
- Convert remaining grid refs to regions format([927b556](https://github.com/scbrown/pixelsrc/commit/927b5565b5c50bbddeb7892678849930aa314fcb))
- Convert tests/demos to regions format([eccd3a0](https://github.com/scbrown/pixelsrc/commit/eccd3a0bcb5450fc75659d8d3a1eaa1522e9c73a))
- *(cli)* Split cli.rs into modular structure([c45ef14](https://github.com/scbrown/pixelsrc/commit/c45ef14115f3d6a3ded550b89b2dd9f735c02a83))
- *(composition)* Split composition.rs into modular structure([9766d9e](https://github.com/scbrown/pixelsrc/commit/9766d9ef8bc115f5a4f325cb4720affabeb06dfd))
- *(analyze)* Split analyze.rs into modular structure([53b1943](https://github.com/scbrown/pixelsrc/commit/53b1943cd4f4a8ebafefd2ff7ccd5dbe713cdc03))
- *(models)* Split models.rs into modular structure([b0082a7](https://github.com/scbrown/pixelsrc/commit/b0082a7f84ae4680643057958b2ceec157ad9235))
- *(lsp)* Split lsp.rs into modular structure([1c5bb6b](https://github.com/scbrown/pixelsrc/commit/1c5bb6bf8f41cf120a1e5b103a2e7b6cbcd9dac6))
- *(registry)* Split registry.rs into modular structure([600daea](https://github.com/scbrown/pixelsrc/commit/600daea0a306a7ffbf680da78b0ba4f684aa1d85))

### Documentation

- *(plan)* Audit phase statuses and document gaps([ce71556](https://github.com/scbrown/pixelsrc/commit/ce71556cf767c21490764be0a09f26a1a84e0322))
- Add Artisan Workflow for autonomous art iteration([a715a75](https://github.com/scbrown/pixelsrc/commit/a715a7547e481a632c6a01b6027f888554a3a423))
- *(plan)* Update phase statuses for 17, 18, 20([fd03ee0](https://github.com/scbrown/pixelsrc/commit/fd03ee0e6112820831da501510b22a1706a30210))
- Reorganize artistic workflow documentation([a0a1c2f](https://github.com/scbrown/pixelsrc/commit/a0a1c2f8d85c4c1523452958f6e3059ebd1d1c47))
- *(plan)* Revise Phase 19 to reflect current implementation([b92f6f9](https://github.com/scbrown/pixelsrc/commit/b92f6f9b5c212b2afaaab773e9325ee24184f5fa))
- *(plan)* Update Phase 19 - mark Color Ramps & Nine-Slice complete([d55ca55](https://github.com/scbrown/pixelsrc/commit/d55ca556cffa812eea19e2d6b674136a6b888ddb))
- *(artistic-workflow)* Add Phase 0 silhouette-first workflow([1c88d35](https://github.com/scbrown/pixelsrc/commit/1c88d358d5463c99b0af3a7624001c91cabff13a))
- *(artistic-workflow)* Expand shape reference with tested details([a355b85](https://github.com/scbrown/pixelsrc/commit/a355b85398a5ff5f3cc1bfd2a654416d86fd13d3))
- *(artistic-workflow)* Add shape diagrams with coordinates([a9b3579](https://github.com/scbrown/pixelsrc/commit/a9b35790909a81949e84cde43b20d3d406bb1465))
- *(book)* Add shape diagrams and silhouette-first workflow([2274b1c](https://github.com/scbrown/pixelsrc/commit/2274b1c61897954b3cf05afc81ebfdeb8437bfed))
- *(transforms)* Clarify CSS vs op-style transform systems([42ecd8c](https://github.com/scbrown/pixelsrc/commit/42ecd8cac04887772e88f0171d87b608576b0e15))
- *(plan)* Add semantic-aware antialiasing phase specification([1dc4e7c](https://github.com/scbrown/pixelsrc/commit/1dc4e7cd5d3a0d3dcd178ffbe658b5593e56290a))
- *(contributing)* Add comprehensive Rust coding standards([7be3802](https://github.com/scbrown/pixelsrc/commit/7be3802558ed2bdc140ae68c092f99356522a23c))
- *(antialias)* Add antialiasing reference chapter([8fb65d7](https://github.com/scbrown/pixelsrc/commit/8fb65d7bade95b3c939b703ef972c1a38392e5a3))
- *(plan)* Add semantic import enhancement plan([7dc4bec](https://github.com/scbrown/pixelsrc/commit/7dc4becfdf2272e788849511b37cdf9db65c5aaa))
- *(plan)* Add Python bindings plan (PyO3 + maturin)([2832c8c](https://github.com/scbrown/pixelsrc/commit/2832c8c3a356b0d32b7a3a9ba969707118600245))
- *(plan)* Add PNG import task (PY-7) to Python bindings plan([b0572f9](https://github.com/scbrown/pixelsrc/commit/b0572f9b3fe69beab5e8f82a12636aa4b36a443d))
- Complete type stubs and documentation for Python bindings (PY-10)([2b7941a](https://github.com/scbrown/pixelsrc/commit/2b7941afd41c8232afd8224f87fcd16a37ddeb6c))
- Regenerate demo documentation to match current tests([18ee6d3](https://github.com/scbrown/pixelsrc/commit/18ee6d355859e6a7a219383da5bf84d29bf22b03))
- Add demo documentation regeneration requirements([21b05ca](https://github.com/scbrown/pixelsrc/commit/21b05ca32c3893b432eb7cfe6b25e2e8a8b48a0f))
- Update plan README to reflect actual implementation status([6547f2d](https://github.com/scbrown/pixelsrc/commit/6547f2d8b0d839e455408d5dea61c405ff177878))
- Add missing phases to plan README — LSP, Python, antialiasing, crates.io([451854d](https://github.com/scbrown/pixelsrc/commit/451854db86f05a9c8b9fd88f68e668e27ac8cb65))
- Add Drafting Table plan docs for scaffold, draw, mask([0e3037b](https://github.com/scbrown/pixelsrc/commit/0e3037be90dc702bd8d8a1a4a5cdd8576e4ecb03))
- Add import system plan with module namespaces and persona analysis([ab9abbd](https://github.com/scbrown/pixelsrc/commit/ab9abbd1c7f8eb3efdf60849a8b4de5a5dbcfce2))
- Add MCP server plan, task breakdown, and implementation beads([aeb08b2](https://github.com/scbrown/pixelsrc/commit/aeb08b2f19ec9fb4badcd8d26af05050cbb9087f))
- Add banner image, lowercase branding, and badges to README([f1345bb](https://github.com/scbrown/pixelsrc/commit/f1345bba1c393209bae0430a24fc0566c435d764))
- Scaffold mdbook with Dracula theme + Intel One Mono([4f785b6](https://github.com/scbrown/pixelsrc/commit/4f785b6f2be5d77e12b7a351d643065225306673))
- Drop an internal forge URL from the Chronicle link — public-repo leak([0220c93](https://github.com/scbrown/pixelsrc/commit/0220c9361c6358936ca9af2bc91f41aa874a7a7a))
- *(examples)* Skull evolution strip, from the January prototypes to now([dd8617d](https://github.com/scbrown/pixelsrc/commit/dd8617d4a664bd6b5821009dac0669b0f416b5bd))
- *(examples)* Animated skull evolution GIF([80f8b50](https://github.com/scbrown/pixelsrc/commit/80f8b501eb5a02ae6cbc114e651281482134cde5))

### Fixed

- Use valid palette colors in hair/hat sprite regions([9a5a140](https://github.com/scbrown/pixelsrc/commit/9a5a1404a4878c275a1ae23838cbd35c99cb372e))
- *(tests)* Update scaffold tests for regions format([573c31f](https://github.com/scbrown/pixelsrc/commit/573c31fa1dcc632a40b43ca930f93f9e8c238d4c))
- Correct dependency direction for large file refactoring epic([c178911](https://github.com/scbrown/pixelsrc/commit/c178911d0de75af04d0a917c9a30172af43021c5))
- *(validate)* Don't warn about missing regions for source sprites([a95833c](https://github.com/scbrown/pixelsrc/commit/a95833c0efc9a05aae3e5550f31318c1a805c517))
- Resolve remaining demo test failures and add missing fixtures([bd9f4b4](https://github.com/scbrown/pixelsrc/commit/bd9f4b49d0c278e368fb3a610c92401dcb60d0a5))
- *(validate)* Resolve var() references before parsing CSS colors([780acd6](https://github.com/scbrown/pixelsrc/commit/780acd6b40b67e2daa99c49ec4d946c21243a333))
- *(hair)* Improve hair coverage on reference-head sprites([4c0cdcc](https://github.com/scbrown/pixelsrc/commit/4c0cdcc3acd702561795e55dfd7d00de41accf89))
- *(shapes)* Fix polygon fill horizontal stripe artifacts([d8c19fb](https://github.com/scbrown/pixelsrc/commit/d8c19fb3aaa8239005fe594f49336dea9da9fbdf))
- *(color)* Replace unwrap() with documented expect() in hex parsing([f1f71e2](https://github.com/scbrown/pixelsrc/commit/f1f71e2cc7092810728fc5c0c7b09fbc8cb9a0bb))
- *(transforms)* Replace unwrap() with documented expect() in production code([ecd7908](https://github.com/scbrown/pixelsrc/commit/ecd790853763a9e2f6b592a0362b2d24056063b7))
- *(ci)* Resolve clippy warnings([ac3799e](https://github.com/scbrown/pixelsrc/commit/ac3799e0948af702c37dc62906ea780882459691))
- *(export)* Handle missing animation frames gracefully in Unity export([4aa70fd](https://github.com/scbrown/pixelsrc/commit/4aa70fd18675272a09c5410288364506833f1c92))
- *(scaffold)* Remove unsafe unwrap() calls in production code([ed225ba](https://github.com/scbrown/pixelsrc/commit/ed225bada0651cdf9bbfff9173eaca9f7797ad54))
- *(analyze)* Replace unsafe unwraps with expect() and total_cmp([a7e9c27](https://github.com/scbrown/pixelsrc/commit/a7e9c272be7166dc00bd928ea971d1eccb9c9319))
- *(lsp)* Replace unwrap() with proper error handling in server([23e0d37](https://github.com/scbrown/pixelsrc/commit/23e0d37201390ddee10a5b22592d00d702be16ce))
- *(build)* Replace Mutex unwrap() with expect() for better error messages([7a9c233](https://github.com/scbrown/pixelsrc/commit/7a9c23399813d888902af3c916c88b5e6a7faef0))
- *(config)* Replace unsafe unwrap() calls with expect() in tests([dde73a0](https://github.com/scbrown/pixelsrc/commit/dde73a0af82678fec451c6d4b63578c9ac621142))
- *(clippy)* Resolve 9 clippy warnings([feaa702](https://github.com/scbrown/pixelsrc/commit/feaa702c85de9b700b5654d9b95e647c18284ab7))
- *(ci)* Exclude python feature from main CI workflow([e4fd8ad](https://github.com/scbrown/pixelsrc/commit/e4fd8add3d4ff65fae92744c87e230587f615127))
- Resolve clippy warnings and compilation errors for CI([837f2a2](https://github.com/scbrown/pixelsrc/commit/837f2a28575d5b77b3739afdcbd0104ce13ec718))
- Remove broken links to deprecated CLI commands in mdbook docs([6a6c5ae](https://github.com/scbrown/pixelsrc/commit/6a6c5ae632394174184dd9a22df095006d71649d))
- Resolve clippy warnings (redundant_closure, collapsible_if)([ec34432](https://github.com/scbrown/pixelsrc/commit/ec34432261538bebc3da6aa4ad7257820c9b19d6))
- Revert grid field, replace TokenGrid with RegionEditor([3ed8555](https://github.com/scbrown/pixelsrc/commit/3ed85559ae88089147a9c62fb18ed541c902ea17))
- Replace v1 grid format with v2 regions in scaffold generators([44d5cfb](https://github.com/scbrown/pixelsrc/commit/44d5cfbeb7420a1d7a03f99f97a9c8acea150304))
- Apply cargo fmt to fix CI formatting check([7e01aff](https://github.com/scbrown/pixelsrc/commit/7e01affb52335b1c060c7765afe4119740ffa9a5))
- Remove unnecessary to_string() to fix clippy CI failure([d8140eb](https://github.com/scbrown/pixelsrc/commit/d8140eb8c8d92def0f57315c1bd24000fa8d64c7))
- Gate MCP integration tests behind `mcp` feature flag([d75c4d5](https://github.com/scbrown/pixelsrc/commit/d75c4d50f63348070c38d91d257d4b735d338430))
- *(docs)* Interactive demos never loaded; wheel and browser-test lanes green([e8e590e](https://github.com/scbrown/pixelsrc/commit/e8e590ede976ba21d39d24f525caffdf4414d797))
- Rasterize circles/ellipses round with pixel-art rounding([3cf32d7](https://github.com/scbrown/pixelsrc/commit/3cf32d729b758fb36ac9d31196765c04d71df78c))
- Credit extends-frame tokens to the inherited palette (no false unused warning)([dcf1342](https://github.com/scbrown/pixelsrc/commit/dcf13423eae2202c47a459c5c8aaf66eae00167b))

### Implement

- REF-8: Test Coverage - Add cargo-llvm-cov, CI integration, coverage targets([561f373](https://github.com/scbrown/pixelsrc/commit/561f37399231bd2609bfe75335f129d73420f15e))

### Merge

- (polecat/pixelsrc-62)([28639d8](https://github.com/scbrown/pixelsrc/commit/28639d82a51d30ab13b91be3e5f771b2fa87a6fc))
- (polecat/pixelsrc-64)([4cb3a34](https://github.com/scbrown/pixelsrc/commit/4cb3a349c2736a5afff435f5478fd2667d39fb6b))
- (polecat/dementus/bobbin-lfgxp)([65315b4](https://github.com/scbrown/pixelsrc/commit/65315b4576e6c9f58452e13dba5d85376df7e792))

### Miscellaneous

- Add .beads/ to gitignore (workspace setup)([5c91449](https://github.com/scbrown/pixelsrc/commit/5c91449e63719efaa453ea5b22cb5bf28dcb0445))
- Update Cargo.lock for pyo3 dependency([bbfa2fa](https://github.com/scbrown/pixelsrc/commit/bbfa2fa695cf1e5ad04e1203e4df0fe337b17922))
- Gitignore Python build artifacts([70f236b](https://github.com/scbrown/pixelsrc/commit/70f236b0d042ad82a1a369c8120bc6a8dcb5b0c1))
- Add root .venv/ to gitignore([3be4e44](https://github.com/scbrown/pixelsrc/commit/3be4e442bea96d9e47a358619d8f299317e43a29))
- Track .beads/metadata.json for dolt.lan:3306 config([e616db9](https://github.com/scbrown/pixelsrc/commit/e616db9a3985a6c005f3f7fc4c650f088b7872f9))
- Add test-generated fixture (minimal_dot_dot.png)([92c44bf](https://github.com/scbrown/pixelsrc/commit/92c44bf5336c51ce25fbad2996af89c1c07fc5cb))
- Add v1 grid deprecation warnings to docs and code([0af98ae](https://github.com/scbrown/pixelsrc/commit/0af98ae1f27d14aaed3e0fe739c7361bb26365bf))
- Fix formatting in cli/mod.rs([93ee086](https://github.com/scbrown/pixelsrc/commit/93ee08697ebff34881ea1e91cd9f17352bd488cd))

### Testing

- Improve Python parse tests with fixtures and coverage (PY-4)([9034141](https://github.com/scbrown/pixelsrc/commit/90341416d2edceb958d59865b2b962aab0700beb))
- Add version metadata tests for Python package (PY-11)([e9ce068](https://github.com/scbrown/pixelsrc/commit/e9ce068b8fd6d6c341e51f66f34faac308eb792f))

### Art

- Add "pxl" text favicon sprite (16x16)([d019384](https://github.com/scbrown/pixelsrc/commit/d01938428d8658700b18e2fede952036ae7dd198))
- Replace P favicon with pxl text favicon([0daa0e9](https://github.com/scbrown/pixelsrc/commit/0daa0e9d6c090e1778600e2a6d243fc8efccc180))
- Iterate skull sprite Round 2 - angular sockets, brow ridge, narrower jaw([7753a11](https://github.com/scbrown/pixelsrc/commit/7753a11dfd724d6a24d203272c2f4021bf709e91))

### Beads

- Add pixelsrc issues for reference-head work([dc333df](https://github.com/scbrown/pixelsrc/commit/dc333dfc55e363deb3f43ca52ac0d03f1e83b7f5))
- Add unwrap cleanup epic with 17 child tasks([51b668c](https://github.com/scbrown/pixelsrc/commit/51b668c933993ae4022255894074c408f25a392b))
- Add Rust idioms epic with 8 child tasks([6a284c1](https://github.com/scbrown/pixelsrc/commit/6a284c12657717a9a8c1f3cd333a873cc73d074c))
- Add for CONTRIBUTING.md Rust standards([dc05525](https://github.com/scbrown/pixelsrc/commit/dc055251d73dc52a2c26d12fc97d5ffdd81fe170))

### Close

- Transforms.rs refactoring complete([8f6956e](https://github.com/scbrown/pixelsrc/commit/8f6956e7570dad8d3d38225aea5cdf42faafdda9))

### Config

- Point beads metadata at per-rig Dolt database([3d3583f](https://github.com/scbrown/pixelsrc/commit/3d3583f4552b6d1a70518d217167823435d173fb))

### Style

- Apply cargo fmt to fix CI formatting check([e9a9279](https://github.com/scbrown/pixelsrc/commit/e9a927934d794cab090334d1dd31e6f43d98949e))
- Cargo fmt over the integrated fork commits([efb161f](https://github.com/scbrown/pixelsrc/commit/efb161fa28832b9a289bef9126194bb3035b226b))

### Sync

- Beads update([df6ae36](https://github.com/scbrown/pixelsrc/commit/df6ae36ded954de30a9a44e0a49a149fac45176a))

<!-- generated by git-cliff; edit cliff.toml, not CHANGELOG.md -->
