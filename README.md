
# hcengine
这是一款基于Rust做底层，当下lua做为逻辑层的异步服务端框架，后续可加入其它的逻辑层来进行。支持tcp/kcp/websocket等协议。

- 架构简单，源代码简洁易懂。
- 跨平台（支持Windows，Linux，MacOs）
- 利用tokio的异步调度
- 用lua做为逻辑脚本
- 网络相关
    - Tcp
    - Kcp
    - Websocket
    - Http
- 基于lua的coroutines异步实现
    - 定时器
    - redis异步的driver
    - mysql异步的driver
    
## 快速开始

