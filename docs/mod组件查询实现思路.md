**AI应当认真阅读此文档，以确保代码生成符合预期**

# 预期效果
实现mod中查询Host中的组件。
## 实现示例
在host中由游戏开发者定义
```rs
#[mod_component(id = "square")] // 允许被mod获取, 设置组件id
#[derive(Component)]
pub struct Square(pub Vec2);

#[mod_component(id = "rect")]
#[derive(Component)]
pub struct Rect(pub IVec2);
```
在sdk中由游戏开发者定义
```rs
#[component(id = "square")] // 确保与host中的id相同
pub struct Square(pub Vec2); // 确保与host中的签名相同

#[component(id = "rect")]
pub struct Rect(pub IVec2);
```
在mod中通过
```rs
use game_sdk::{Square, Rect};

for (square, rect) in query!(Square, Rect) {}
```
## 可能的后期拓展
### 实现query_mut!宏的可变查询
允许mod开发者通过
```rs
let mut rects = query_mut(Square);
```

# 核心难点
在WASM中无法直接访问Host中的内存, 且双端无法直接传递复杂结构体。
在WASM获取到组件并使用完成后，需要将内存释放避免内存泄漏。

# 基础实现思路
## `mod_component`的实现
在`modruntime_macros`中定义该宏。该宏应为过程宏，在宏展开后，应当实现这些功能：
- 为`Component`实现`serde`和`bincode`的序列化和反序列化
- 在注册表中注册组件(可以使用`linkme`)
## `sdk`中`component`实现
仅用于在mod中在mod查询时将复杂结构体转换为id, 传递给Host进行查询
在sdk宏定义组件签名方便在Host传回组件后将结构体反序列化
## `query`宏实现
将其中的结构体转换为id, 并获取组件。
