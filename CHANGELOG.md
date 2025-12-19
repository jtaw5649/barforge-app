# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add author profiles and reviews to desktop app ([#22](https://github.com/jtaw5649/waybar-manager/pull/22)) ([7b8b4ad](https://github.com/jtaw5649/waybar-manager/commit/7b8b4ad9e3c70a2dddfd8aace5462b9b95227b5b))

- V0.3.1 release with security, UI redesign, and shared types ([#12](https://github.com/jtaw5649/waybar-manager/pull/12)) ([ea88fc7](https://github.com/jtaw5649/waybar-manager/commit/ea88fc76839d732de9abede0efe2abefb800589b))

- Add script content inspection for security analysis ([f204829](https://github.com/jtaw5649/waybar-manager/commit/f2048290165ec0d85724ea15e4362c9bb405ce47))

- Add GitHub URL validation for module downloads ([93975b7](https://github.com/jtaw5649/waybar-manager/commit/93975b7485c7389f6ffa1c53d9d43886a8bac800))

- Improve omarchy theme detection using alacritty.toml ([0f516c4](https://github.com/jtaw5649/waybar-manager/commit/0f516c4e2e444435cf49dddc27634a145cea2a18))

- Add refresh registry button and last-refreshed timestamp ([eabaf59](https://github.com/jtaw5649/waybar-manager/commit/eabaf5994819a03c983b40a184a20e9559f831db))

- Add preferences button and apply prefs to module config ([418ac23](https://github.com/jtaw5649/waybar-manager/commit/418ac2389d9844f12958d28228e9801ded1170cb))

- Implement module config merging, CSS injection, and script permissions ([2108618](https://github.com/jtaw5649/waybar-manager/commit/2108618b0ba73721bd8f3e1e3a293ccd61f83289))

- Implement module management with waybar integration ([3ce2c9d](https://github.com/jtaw5649/waybar-manager/commit/3ce2c9d418c896e6bfc816c81204e4f1f42f4933))

- Complete iced 0.14 migration with enhanced UI features ([5584fc6](https://github.com/jtaw5649/waybar-manager/commit/5584fc66c184315d1efd2fe0a1cb3cd8c7e49974))

- Initial release of waybar-manager ([6489e0a](https://github.com/jtaw5649/waybar-manager/commit/6489e0acbb4fe9c4ed86e8e2113cf7806a4167e3))


### Fixed

- Add checks:write permission to security workflow ([#23](https://github.com/jtaw5649/waybar-manager/pull/23)) ([2e8dab1](https://github.com/jtaw5649/waybar-manager/commit/2e8dab1a2cc657dba6348fe4c9b68e5e76cb4d63))

- Eliminate silent failures with proper error logging ([c161c03](https://github.com/jtaw5649/waybar-manager/commit/c161c03041f711caaa5e49f52a580da6b6d34c6a))

- Show and activate omarchy theme button correctly ([2b3f75d](https://github.com/jtaw5649/waybar-manager/commit/2b3f75de2a4e7bf4d38035ba6af2f964b4d64297))


### Other

- Bump versions to 0.4.0 and disable crates.io publishing ([#25](https://github.com/jtaw5649/waybar-manager/pull/25)) ([fbb69d0](https://github.com/jtaw5649/waybar-manager/commit/fbb69d036b5a40ecccf9c804956bdd836f0407fc))

- Bump sigstore from 0.10.0 to 0.13.0 ([b076ebc](https://github.com/jtaw5649/waybar-manager/commit/b076ebc9801ed658a46706a3f826d49b060127ee))

- Bump toml from 0.8.23 to 0.9.8 ([47abc6b](https://github.com/jtaw5649/waybar-manager/commit/47abc6b4404a67002a2afa7294edea8395d03465))

- Bump ts-rs from 10.1.0 to 11.1.0 ([d45bf60](https://github.com/jtaw5649/waybar-manager/commit/d45bf6083a9a996fec6f2050f38c9e4a96f0d93a))

- Bump criterion from 0.5.1 to 0.8.1 ([53c63bb](https://github.com/jtaw5649/waybar-manager/commit/53c63bb35c2cb82459cdf180b154c85da86404cb))

- Bump tracing in the minor-and-patch group ([6579acb](https://github.com/jtaw5649/waybar-manager/commit/6579acb5d6825eeec367abb68aeea5faddcb7cda))

- Bump actions/upload-artifact from 4 to 6 ([25be73c](https://github.com/jtaw5649/waybar-manager/commit/25be73c1ca620f1ad00d032406f4c431e8328ec3))

- Bump actions/checkout from 4 to 6 ([f2668bd](https://github.com/jtaw5649/waybar-manager/commit/f2668bdcf4ad96c391a575952348f39971ae5605))

- Bump codecov/codecov-action from 4 to 5 ([#16](https://github.com/jtaw5649/waybar-manager/pull/16)) ([22b6c62](https://github.com/jtaw5649/waybar-manager/commit/22b6c62edf766c8ed9065aa474c064b4010dc474))

- Bump actions/download-artifact from 4 to 7 ([#13](https://github.com/jtaw5649/waybar-manager/pull/13)) ([9f541ad](https://github.com/jtaw5649/waybar-manager/commit/9f541ad278faba959300331bbb361dde25f3110a))

- Convert remaining Message UUID fields to ModuleUuid ([6828375](https://github.com/jtaw5649/waybar-manager/commit/6828375bb6a3d91dd1312ab7cb189eefa77fb744))

- Convert Message enum UUID fields to ModuleUuid type ([d57d5dc](https://github.com/jtaw5649/waybar-manager/commit/d57d5dc38bc319f9671d2cc00b973b550c5e590c))

- Remove dead code and DRY PickListColors ([f852f63](https://github.com/jtaw5649/waybar-manager/commit/f852f6358b5f14ddbb9d8835023e87380f7a1d08))

- Code audit cleanup and schema alignment ([7a23e04](https://github.com/jtaw5649/waybar-manager/commit/7a23e0433cd57cd2795f45e1216231a76eed5b24))

- Comprehensive security and code quality improvements ([ea97166](https://github.com/jtaw5649/waybar-manager/commit/ea971666494d3dace3d4f97423ed51ecd5f5efd8))

- Bump version to 0.0.4 ([ec68f27](https://github.com/jtaw5649/waybar-manager/commit/ec68f2751afa635738dfbd28bc74b1c3830dd918))

- Remove waybar version compatibility checking ([1fe4185](https://github.com/jtaw5649/waybar-manager/commit/1fe41855aae813a3f169c227a4ba21fd7c48065b))

- Migrate UI from GTK4 to iced 0.14 ([e2cdaa4](https://github.com/jtaw5649/waybar-manager/commit/e2cdaa45cfe18b61035d5e5e2e1f44b7269ef814))

- Add app icon to README ([27b6299](https://github.com/jtaw5649/waybar-manager/commit/27b6299e69f6087f73eaef7d13ad2e2b83df7442))

