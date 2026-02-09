# PDFMill 架构文档

## 项目概述

PDFMill 是一个用 Rust 编写的高性能 PDF 转换服务，受 Gotenberg 启发。核心特性是**智能路由** - 根据文件扩展名自动选择合适的转换引擎，提供统一的 API 接口。

## 设计理念

### 1. 智能路由优先

与 Gotenberg 需要为不同文件类型调用不同端点不同，PDFMill 只提供一个 `/convert` 端点：

```
Gotenberg 方式:
- POST /forms/chromium/convert/html
- POST /forms/libreoffice/convert
- POST /forms/pdfengines/merge

PDFMill 方式:
- POST /convert (自动路由到正确的引擎)
```

### 2. 零配置启动

服务启动时自动检测可用引擎，无需手动配置。用户只需安装需要的转换工具（Chrome、LibreOffice、ImageMagick）。

### 3. 高性能异步

使用 Tokio 异步运行时和 Axum web 框架，支持高并发请求处理。

## 技术栈

- **Web 框架**: Axum 0.7
- **异步运行时**: Tokio
- **HTTP 客户端**: Reqwest
- **错误处理**: thiserror + anyhow
- **日志**: tracing + tracing-subscriber

## 项目结构

```
pdfmill/
├── src/
│   ├── main.rs                 # 服务入口
│   ├── error.rs                # 错误类型定义
│   ├── engines/                # 转换引擎实现
│   │   ├── mod.rs             # 引擎 trait 定义
│   │   ├── chromium.rs        # HTML/Markdown 引擎
│   │   ├── libreoffice.rs     # Office 文档引擎
│   │   └── image.rs           # 图片转换引擎
│   ├── router/                 # 智能路由
│   │   └── mod.rs             # 路由逻辑
│   └── handlers/               # HTTP 处理器
│       └── mod.rs             # 请求处理
├── examples/                   # 示例文件
│   ├── sample.html
│   └── sample.md
├── Cargo.toml                  # Rust 依赖
├── Dockerfile                  # Docker 镜像
├── docker-compose.yml          # Docker Compose 配置
├── test.sh                     # 测试脚本
├── README.md                   # 主文档
├── QUICKSTART.md              # 快速开始
└── ARCHITECTURE.md            # 本文档
```

## 核心模块

### 1. 智能路由器 (SmartRouter)

**文件**: `src/router/mod.rs`

**职责**:
- 维护所有可用的转换引擎
- 根据文件扩展名选择合适的引擎
- 处理引擎不可用的情况
- 提供格式支持查询

**关键方法**:
```rust
pub async fn find_engine_for_file(&self, path: &Path) -> Result<Arc<dyn ConvertEngine>>
pub fn supported_extensions(&self) -> Vec<String>
pub fn is_extension_supported(&self, ext: &str) -> bool
```

**工作流程**:
1. 从文件名提取扩展名
2. 查找支持该扩展名的所有引擎
3. 检查引擎是否可用（依赖已安装）
4. 返回第一个可用的引擎

### 2. 转换引擎 (ConvertEngine Trait)

**文件**: `src/engines/mod.rs`

**Trait 定义**:
```rust
#[async_trait]
pub trait ConvertEngine: Send + Sync {
    fn engine_type(&self) -> EngineType;
    fn supports_extension(&self, ext: &str) -> bool;
    fn supported_extensions(&self) -> Vec<&'static str>;
    async fn is_available(&self) -> bool;
    async fn convert(&self, input_path: &Path, options: &ConvertOptions) -> Result<ConvertResult>;
}
```

#### 2.1 ChromiumEngine

**文件**: `src/engines/chromium.rs`

**支持格式**: HTML, HTM, XHTML, MD, Markdown

**实现细节**:
- 使用 Chrome/Chromium 的 headless 模式
- 调用 `--print-to-pdf` 参数生成 PDF
- Markdown 先转换为 HTML 再处理
- 支持背景打印、页面大小等选项

**依赖**: Chrome/Chromium 浏览器

#### 2.2 LibreOfficeEngine

**文件**: `src/engines/libreoffice.rs`

**支持格式**: DOC, DOCX, XLS, XLSX, PPT, PPTX, ODT, ODS, ODP, RTF

**实现细节**:
- 使用 LibreOffice 的命令行接口
- 调用 `soffice --headless --convert-to pdf`
- 支持所有 Office 文档格式

**依赖**: LibreOffice

#### 2.3 ImageEngine

**文件**: `src/engines/image.rs`

**支持格式**: JPG, JPEG, PNG, GIF, BMP, TIFF, TIF, WEBP

**实现细节**:
- 使用 ImageMagick 的 convert 命令
- 支持自定义页面大小

**依赖**: ImageMagick

### 3. HTTP 处理器 (Handlers)

**文件**: `src/handlers/mod.rs`

#### 3.1 convert_handler

**端点**: `POST /convert`

**处理流程**:
1. 解析 multipart/form-data 请求
2. 提取文件和可选参数
3. 保存文件到临时目录
4. 通过 SmartRouter 找到合适的引擎
5. 调用引擎执行转换
6. 返回生成的 PDF 文件

**参数**:
- `file` (必需): 要转换的文件
- `landscape` (可选): 横向模式
- `printBackground` (可选): 打印背景
- `pageWidth` (可选): 页面宽度
- `pageHeight` (可选): 页面高度
- `pdfFormat` (可选): PDF 格式

#### 3.2 health_handler

**端点**: `GET /health`

简单的健康检查端点，返回服务状态。

#### 3.3 info_handler

**端点**: `GET /info` 或 `GET /`

返回服务信息，包括：
- 服务名称和版本
- 支持的文件格式列表
- API 端点文档

### 4. 错误处理 (Error)

**文件**: `src/error.rs`

**错误类型**:
```rust
pub enum AppError {
    UnsupportedFormat(String),      // 不支持的格式
    NoFileProvided,                  // 未提供文件
    ConversionFailed(String),        // 转换失败
    EngineNotAvailable(String),      // 引擎不可用
    InvalidRequest(String),          // 无效请求
    IoError(std::io::Error),        // IO 错误
    Internal(String),                // 内部错误
}
```

每种错误都会映射到合适的 HTTP 状态码和用户友好的错误消息。

## 数据流

### 完整的转换流程

```
1. HTTP Request
   ↓
2. Axum Router → convert_handler
   ↓
3. Parse multipart form data
   ↓ (file + options)
4. Save to temp file
   ↓
5. SmartRouter.find_engine_for_file()
   ↓ (extract extension)
6. Find matching engine
   ↓ (check availability)
7. Engine.convert()
   ↓
   ├─ ChromiumEngine → Chrome CLI
   ├─ LibreOfficeEngine → soffice CLI
   └─ ImageEngine → convert CLI
   ↓
8. Read generated PDF
   ↓
9. HTTP Response with PDF
   ↓
10. Cleanup temp files
```

## 并发模型

### Tokio 异步运行时

```
┌─────────────────────────────┐
│     Tokio Runtime           │
│  (多线程工作调度器)            │
└──────────┬──────────────────┘
           │
    ┌──────┴──────┐
    ▼             ▼
┌────────┐  ┌────────┐
│ Worker │  │ Worker │  ... (多个工作线程)
│ Thread │  │ Thread │
└────────┘  └────────┘
```

### 请求处理

每个请求都是一个独立的异步任务：

```rust
async fn convert_handler(...) -> Result<Response> {
    // 1. 异步解析请求
    let file_data = parse_multipart().await?;
    
    // 2. 异步文件操作
    tokio::fs::write(&path, &data).await?;
    
    // 3. 异步执行转换
    let result = engine.convert(&path, &options).await?;
    
    // 4. 返回响应
    Ok(response)
}
```

## 性能考虑

### 1. 零拷贝

使用 `Bytes` 类型避免不必要的内存拷贝。

### 2. 流式处理

大文件使用流式读写，减少内存占用。

### 3. 临时文件管理

使用 `tempfile` crate 自动清理临时文件。

### 4. 异步 I/O

所有 I/O 操作都是异步的，不会阻塞工作线程。

## 安全性

### 1. 文件隔离

每个转换请求使用独立的临时目录。

### 2. 扩展名验证

只接受白名单中的文件扩展名。

### 3. 进程隔离

转换工具（Chrome、LibreOffice）在独立进程中运行。

### 4. 资源限制

通过 Tokio 的并发控制限制同时执行的转换任务数。

## 可扩展性

### 添加新引擎

1. 实现 `ConvertEngine` trait
2. 在 `SmartRouter::new()` 中注册引擎
3. 完成！无需修改其他代码

示例:
```rust
// src/engines/pandoc.rs
pub struct PandocEngine { ... }

#[async_trait]
impl ConvertEngine for PandocEngine {
    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "docx" | "epub" | "rst")
    }
    // ... 实现其他方法
}

// src/router/mod.rs
let engines: Vec<Arc<dyn ConvertEngine>> = vec![
    Arc::new(ChromiumEngine::new()),
    Arc::new(LibreOfficeEngine::new()),
    Arc::new(ImageEngine::new()),
    Arc::new(PandocEngine::new()),  // 新引擎
];
```

## 部署

### Docker 部署

多阶段构建优化镜像大小：

1. **Builder stage**: 编译 Rust 代码
2. **Runtime stage**: 只包含二进制和运行时依赖

### 水平扩展

可以运行多个 PDFMill 实例，通过负载均衡器分发请求：

```
         ┌──────────────┐
         │  Load        │
         │  Balancer    │
         └──────┬───────┘
                │
        ┌───────┼───────┐
        ▼       ▼       ▼
    ┌──────┐ ┌──────┐ ┌──────┐
    │PDFMill│ │PDFMill│ │PDFMill│
    │  :3000│ │  :3001│ │  :3002│
    └──────┘ └──────┘ └──────┘
```

## 监控和日志

### 日志级别

```bash
# 开发环境
RUST_LOG=pdfmill=debug

# 生产环境
RUST_LOG=pdfmill=info
```

### 关键日志点

- 服务启动和引擎检测
- 每个转换请求的引擎选择
- 转换成功/失败
- 错误详情

## 未来改进

1. **缓存**: 对相同输入进行结果缓存
2. **队列**: 使用消息队列处理大量请求
3. **Webhook**: 异步处理完成后回调
4. **批量转换**: 一次请求转换多个文件
5. **PDF 操作**: 合并、分割、压缩等
6. **指标**: Prometheus 监控指标
7. **gRPC**: 支持 gRPC 协议

## 与 Gotenberg 对比

| 特性 | PDFMill | Gotenberg |
|-----|---------|-----------|
| 语言 | Rust | Go |
| 路由方式 | 智能自动 | 显式指定 |
| API 设计 | 单一端点 | 多个端点 |
| 配置 | 零配置 | 需配置 |
| 性能 | 更高（Rust + 异步） | 高 |
| 内存安全 | 编译期保证 | 运行时 GC |
| 二进制大小 | 更小 | 较大 |

## 总结

PDFMill 通过智能路由简化了 PDF 转换服务的使用，提供了清晰的架构和良好的扩展性。其核心优势在于：

1. **简单**: 单一 API 端点，自动选择引擎
2. **高效**: Rust + 异步实现高性能
3. **灵活**: 易于添加新的转换引擎
4. **可靠**: 完善的错误处理和日志

这使得 PDFMill 成为需要 PDF 转换服务的应用的理想选择。
