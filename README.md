# Bevy Wasm Mod

English | [中文简体](README-zh.md)

Easily add mod support to your Bevy game using WebAssembly (Wasm)!

| Crate Name | Functionality |
| -- | -- |
| bevy_modapi | API provided for mod developers |
| bevy_modruntime | Interface for game developers to integrate mod APIs |
| bevy_modsdk | Interface for game developers to develop SDKs |

*Note: This project is currently in early stages, has not been thoroughly tested, is highly unstable, and has not undergone any optimization.*

## Version Compatibility
| wasm_mod version | bevy version |
| -- | -- |
| 0.16 | 0.16 |

## Quick Start
- Check out the [hello_world](examples/hello_world/README.md) example
- Read the [Quick Start Guide](docs/quick_start.md)

## Project Progress & Roadmap
- [x] Add Systems from mods
- [x] Query game Components from mods
- [ ] Modify game Components from mods (or mutable queries)
- [x] Add entities from mods
- [x] Read game Resources from mods
- [ ] Modify game Resources from mods (or mutable access)
- [ ] Receive game events in mods
- [x] Add assets (e.g., images) to the game from mods
- [ ] Hot loading/unloading of mods
- [ ] Provide toolchain support for mod developers

## Open Source License
This project is open source. Unless otherwise stated, it is licensed under the `Apache-2.0` license.

## Contributions
Contributions to this project are welcome. Unless explicitly stated otherwise, all contributed code will be licensed under the `Apache-2.0` license by default.