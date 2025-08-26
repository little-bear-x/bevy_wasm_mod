**AI应当认真阅读此文档，以确保代码生成符合预期**

# 预期效果
实现mod中查询Host中的Resource。
## 实现示例
在host中由游戏开发者定义
```rs
#[mod_resource(id = "player")] // 允许被mod获取, 设置组件id
#[derive(Resource)]
pub struct Player(pub Vec<IVec2>);
```
在sdk中由游戏开发者定义
```rs
#[component(id = "player")] // 确保与host中的id相同
pub struct Player(pub Vec<IVec2>); // 确保与host中的签名相同
```
在mod中通过
```rs
use game_sdk::{Player};

let player = res!(Player);
```
## 可能的后期拓展
### 实现res_mut!宏的可变查询
允许mod开发者通过
```rs
let mut player = res_mut!(Player);
```

# 参考思路
参考query组件查询实现思路
