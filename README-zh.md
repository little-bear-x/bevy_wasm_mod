# Bevy Wasm Mod

[English](README.md) | 中文简体

使用wasm为你的bevy游戏轻松添加mod功能!

| crate 名称 | 功能 |
| -- | -- |
| bevy_modapi | 为mod开发者提供的api |
| bevy_modruntime | 为游戏开发者提供接入api的接口 |
| bevy_modsdk | 为游戏开发者提供的开发sdk的接口 |

*note: 此项目目前处于早期阶段，未经充分测试，极不稳定且没有进行任何优化*

## 版本说明
| wasm_mod版本 | bevy版本 |
| -- | -- |
| 0.16 | 0.16 |

## 快速开始
- 查看[hello_world](examples/hello_world/README.md)示例
- 查看[快速开始文档](docs/zh/快速开始.md)（必读）

## 项目进度&规划
- [x] 从mod中添加System
- [x] 从mod中查询游戏Component
- [ ] 从mod中修改游戏Component（或可变查询）
- [x] 在mod中添加实体
- [x] 从mod中读取游戏Resource
- [ ] 从mod中修改游戏Resource（或可变修改）
- [ ] 从mod中获取游戏事件
- [x] 在mod中为游戏添加资产（如图片等）
- [ ] 热加载/卸载mod
- [ ] 为mod开发者提供工具链

## 开源许可
此项目是开源项目，除非另有说明，否则使用`Apache-2.0`作为许可证。

## 贡献
欢迎为此项目进行贡献。除非您有明确说明，否则您的代码都将默认使用`Apache-2.0`许可证

