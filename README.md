# PDFMill

🚀 一个用 Rust 编写的高性能 PDF 转换服务，灵感来自 Gotenberg

## ✨ 特性

- **智能路由**: 根据文件扩展名自动选择转换引擎，无需显式指定
- **统一 API**: 只有一个 `/convert` 端点处理所有文件类型
- **多引擎支持**:
  - 📄 **Chromium**: HTML, Markdown → PDF
  - 📊 **LibreOffice**: Word, Excel, PowerPoint, ODT → PDF
  - 🖼️ **ImageMagick**: JPG, PNG, GIF, BMP → PDF
- **高性能**: Rust + Tokio 异步架构
- **零配置**: 自动检测可用引擎
- **RESTful API**: 简单易用的 HTTP 接口

## 📦 安装依赖

### macOS

```bash
# Chrome (用于 HTML/Markdown)
brew install --cask google-chrome

# LibreOffice (用于 Office 文档)
brew install --cask libreoffice

# ImageMagick (用于图片)
brew install imagemagick
```

### Linux

```bash
# Chrome/Chromium
sudo apt install chromium-browser

# LibreOffice
sudo apt install libreoffice

# ImageMagick
sudo apt install imagemagick
```

## 🚀 快速开始

### 构建和运行

```bash
# 克隆仓库
git clone <your-repo>
cd pdfmill

# 构建
cargo build --release

# 运行
cargo run --release
```

服务将在 `http://0.0.0.0:3000` 启动

### 环境变量

```bash
# 自定义监听地址
export PDFMILL_ADDR=0.0.0.0:8080

# 日志级别
export RUST_LOG=pdfmill=debug
```

## 📖 API 使用

### 转换文件 (智能路由)

**所有文件类型使用同一个端点！**

```bash
# 转换 HTML
curl -X POST http://localhost:3000/convert \
  -F "file=@document.html" \
  -o output.pdf

# 转换 Word 文档
curl -X POST http://localhost:3000/convert \
  -F "file=@document.docx" \
  -o output.pdf

# 转换图片
curl -X POST http://localhost:3000/convert \
  -F "file=@image.jpg" \
  -o output.pdf

# 转换 Markdown
curl -X POST http://localhost:3000/convert \
  -F "file=@readme.md" \
  -o output.pdf
```

### 可选参数

```bash
# 横向模式
curl -X POST http://localhost:3000/convert \
  -F "file=@document.html" \
  -F "landscape=true" \
  -o output.pdf

# 打印背景 (HTML)
curl -X POST http://localhost:3000/convert \
  -F "file=@document.html" \
  -F "printBackground=true" \
  -o output.pdf

# 自定义页面大小
curl -X POST http://localhost:3000/convert \
  -F "file=@document.html" \
  -F "pageWidth=8.5in" \
  -F "pageHeight=11in" \
  -o output.pdf
```

### 其他端点

```bash
# 健康检查
curl http://localhost:3000/health

# 服务信息和支持的格式
curl http://localhost:3000/info
```

## 🎯 支持的格式

| 格式类型 | 扩展名 | 引擎 |
|---------|--------|------|
| HTML/Web | .html, .htm, .xhtml | Chromium |
| Markdown | .md, .markdown | Chromium |
| Word | .doc, .docx | LibreOffice |
| Excel | .xls, .xlsx | LibreOffice |
| PowerPoint | .ppt, .pptx | LibreOffice |
| OpenDocument | .odt, .ods, .odp | LibreOffice |
| RTF | .rtf | LibreOffice |
| Images | .jpg, .jpeg, .png, .gif, .bmp, .tiff, .webp | ImageMagick |

## 🏗️ 架构设计

```
┌─────────────────────────────────────────┐
│         HTTP Request (/convert)         │
│          + file (multipart)             │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│          Smart Router                   │
│  (根据文件扩展名自动选择引擎)            │
└────────────────┬────────────────────────┘
                 │
        ┌────────┼────────┐
        ▼        ▼        ▼
   ┌────────┐ ┌────────┐ ┌────────┐
   │Chromium│ │LibreOff│ │ Image  │
   │ Engine │ │  ice   │ │ Engine │
   │        │ │ Engine │ │        │
   └───┬────┘ └───┬────┘ └───┬────┘
       │          │          │
       └──────────┼──────────┘
                  ▼
           ┌─────────────┐
           │  PDF Output │
           └─────────────┘
```

### 核心组件

1. **Smart Router** (`src/router/mod.rs`)
   - 自动检测可用引擎
   - 根据文件扩展名路由到合适的引擎
   - 处理引擎不可用的情况

2. **Engines** (`src/engines/`)
   - `chromium.rs`: HTML/Markdown 转换
   - `libreoffice.rs`: Office 文档转换
   - `image.rs`: 图片转换
   - 统一的 `ConvertEngine` trait

3. **Handlers** (`src/handlers/mod.rs`)
   - HTTP 请求处理
   - Multipart 表单解析
   - 响应生成

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test router

# 带详细输出
cargo test -- --nocapture
```

## 🐳 Docker 部署

```dockerfile
# Dockerfile 示例
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    chromium \
    libreoffice \
    imagemagick \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pdfmill /usr/local/bin/
EXPOSE 3000
CMD ["pdfmill"]
```

## 🔧 配置

可以通过环境变量配置引擎路径：

```bash
export CHROME_PATH="/path/to/chrome"
export SOFFICE_PATH="/path/to/soffice"
export CONVERT_PATH="/path/to/convert"
```

## 📊 性能

- 异步处理，支持高并发
- 零拷贝文件处理
- 自动资源清理
- 最小内存占用

## 🤝 与 Gotenberg 的对比

| 特性 | PDFMill | Gotenberg |
|-----|---------|----------|
| 语言 | Rust | Go |
| 智能路由 | ✅ 自动 | ❌ 需要指定 |
| API 端点 | 1 个统一端点 | 多个专用端点 |
| 性能 | 更高 | 高 |
| 部署 | 二进制/Docker | Docker |

## 📝 开发计划

- [ ] 添加 PDF 合并功能
- [ ] 支持批量转换
- [ ] Webhook 回调
- [ ] 更多 PDF 选项 (加密、水印等)
- [ ] 性能监控和指标
- [ ] gRPC API

## 📄 许可证

MIT License

## 🙏 致谢

灵感来自 [Gotenberg](https://gotenberg.dev/) 项目
