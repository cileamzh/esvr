# Rust File Bridge

一个基于 Axum 框架实现的轻量级文件映射服务器。支持将本地物理路径映射到 Web 虚拟路径，并具备自动识别 MIME 类型和列出目录功能。

## 🚀 功能特性

- **虚拟路径映射**：通过配置文件动态映射多个物理目录。
- **智能响应**：
  - **文件**：自动识别 Content-Type 并以异步流（Stream）方式返回。
  - **目录**：返回该目录下所有文件名的 JSON 列表。
- **异步高性能**：基于 Tokio 运行时，支持高并发访问。
- **内存优化**：锁粒度优化，在文件 IO 期间不阻塞全局状态读取。


```
⚙️ 配置文件 (config.json)
在程序运行目录下创建 config.json，格式如下：

JSON
{
  "host": "127.0.0.1:3000",
 
    //virt 是虚拟路径，real是本地路径
  "mounts": [
    {
      "virt": "static",
      "real": "C:/Users/Documents/Assets"
    },
    {
      "virt": "videos",
      "real": "/var/data/media"
    }
  ]
}
🛠️ 核心代码逻辑解析
```
