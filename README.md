# 项目介绍&最终愿景
这个项目希望为Bevy引擎提供完整Mod支持，允许Mod开发者通过简单的宏标记，简单的编写游戏mod，同时让游戏开发者能够快速为自身游戏集成Mod。

*note: 此项目目前极不稳定且没有进行任何优化*

项目将考虑（优先级由高至低）：
- Mod能够正常运行符合预期
- 让Mod开发者尽可能少写代码
- 让游戏本体开发者尽可能少写代码

## 项目进度
- [ x ] 宏解析
- [ x ] 添加mod系统
- [ x ] mod系统对host组件的查询
- [ ] mod系统对host组件的修改
- [ x ] mod读取host资源
- [ ] mod修改host资源
- [ ] host与mod共享事件
- [ x ] 通过mod添加资产(如图片等)
- [ ] 为mod开发者提供工具链

# 快速开始
尝试编译运行`example/hello_world`中的测试示例

## 编译mod
在`game_mod`目录下执行

```sh
cargo +1.86 build --target wasm32-wasip1
```

*note: rust的最新stable版本目前似乎无法编译到wasm32-wasip1目标平台，这里将暂时使用1.86进行编译，可能会被修复。*

## 修改host中mod的文件路径
修改`host/src/main.rs`文件中的mod的wasm路径为正确路径

## 编译游戏本体
在`host`目录下执行

```sh
cargo run
```

*note: 支持使用动态链接*
