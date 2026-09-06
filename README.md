# NoyAcg Aidoku 全年龄图源

为 [NoyAcg](https://noymanga.com/) 制作的 Aidoku `.aix` 图源，只读取全年龄漫画。

## 安装

从 [Releases](https://github.com/asuharayuuki/aidoku-noyacg-source/releases/latest) 下载
`zh.noymanga-safe.aix`，再使用 Aidoku 打开并安装。仓库内也保留了
[当前构建产物](dist/zh.noymanga-safe.aix)。要求 Aidoku 0.7 或更高版本。

安全限制：

- 所有 API 请求固定携带 `allow-adult: false`。
- 列表、搜索、详情和阅读入口都会检查成人内容标记。
- 登录用户名、密码不写入源码或安装包，需要在 Aidoku 的图源设置中填写。

## 构建

安装 Rust nightly 和 `wasm32-unknown-unknown` target 后，在 PowerShell 运行。当前 `aidoku-cli` 为可选依赖；检测到时构建脚本会优先使用它打包。

```powershell
./build.ps1
```

产物位于 `dist/zh.noymanga-safe.aix`。

当前构建产物 SHA-256：

```text
BD1E3DAED8CAF01306B19B21827519039B834A2D57C617022F8C93052500DB0E
```
