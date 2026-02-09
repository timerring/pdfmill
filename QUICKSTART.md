# PDFMill 快速开始指南

## 前提条件

在开始之前，请确保已安装：

### 1. Rust 工具链

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重启终端或运行
source $HOME/.cargo/env

# 验证安装
cargo --version
```

### 2. 转换引擎（至少安装一个）

#### macOS

```bash
# Chrome (推荐 - 用于 HTML/Markdown)
brew install --cask google-chrome

# LibreOffice (用于 Office 文档)
brew install --cask libreoffice

# ImageMagick (用于图片)
brew install imagemagick
```

#### Linux (Debian/Ubuntu)

```bash
# Chrome/Chromium
sudo apt update
sudo apt install chromium-browser

# LibreOffice
sudo apt install libreoffice

# ImageMagick
sudo apt install imagemagick
```

## 构建和运行

### 方式 1: 本地运行

```bash
# 1. 克隆或进入项目目录
cd pdfmill

# 2. 构建项目
cargo build --release

# 3. 运行服务
cargo run --release
```

服务将在 `http://localhost:3000` 启动。

### 方式 2: Docker 运行

```bash
# 使用 docker-compose (推荐)
docker-compose up -d

# 或者使用 docker build
docker build -t pdfmill .
docker run -p 3000:3000 pdfmill
```

## 测试服务

### 1. 检查服务状态

```bash
# 健康检查
curl http://localhost:3000/health

# 获取服务信息和支持的格式
curl http://localhost:3000/info
```

### 2. 转换文件

```bash
# 转换 HTML 文件
curl -X POST http://localhost:3000/convert \
  -F "file=@examples/sample.html" \
  -o output.pdf

# 转换 Markdown 文件
curl -X POST http://localhost:3000/convert \
  -F "file=@examples/sample.md" \
  -o output.pdf

# 使用可选参数
curl -X POST http://localhost:3000/convert \
  -F "file=@examples/sample.html" \
  -F "landscape=true" \
  -F "printBackground=true" \
  -o output.pdf
```

### 3. 运行自动化测试

```bash
# 运行测试脚本
./test.sh

# 测试远程服务
./test.sh http://your-server:3000
```

## 支持的文件格式

**所有文件类型都使用同一个 `/convert` 端点！**

| 格式 | 扩展名 | 所需引擎 |
|------|--------|---------|
| HTML | .html, .htm, .xhtml | Chrome/Chromium |
| Markdown | .md, .markdown | Chrome/Chromium |
| Word | .doc, .docx | LibreOffice |
| Excel | .xls, .xlsx | LibreOffice |
| PowerPoint | .ppt, .pptx | LibreOffice |
| OpenDocument | .odt, .ods, .odp | LibreOffice |
| Images | .jpg, .png, .gif, .bmp, .tiff, .webp | ImageMagick |

## API 参数

### 必需参数

- `file`: 要转换的文件（multipart/form-data）

### 可选参数

- `landscape`: 布尔值，横向模式 (true/false)
- `printBackground`: 布尔值，打印背景图形 (true/false, 仅 HTML)
- `pageWidth`: 页面宽度（如 "8.5in", "210mm"）
- `pageHeight`: 页面高度（如 "11in", "297mm"）
- `pdfFormat`: PDF 格式（如 "PDF/A-1b"）

## 环境变量

```bash
# 自定义监听地址
export PDFMILL_ADDR=0.0.0.0:8080

# 日志级别
export RUST_LOG=pdfmill=debug

# 自定义引擎路径（可选）
export CHROME_PATH="/path/to/chrome"
export SOFFICE_PATH="/path/to/soffice"
export CONVERT_PATH="/path/to/convert"
```

## 常见问题

### Q: 服务启动时显示某些引擎不可用？

**A:** 这是正常的。PDFMill 会自动检测可用的引擎。只需确保你想使用的格式对应的引擎已安装即可。

### Q: 转换 HTML 时没有背景色或图片？

**A:** 添加 `printBackground=true` 参数：

```bash
curl -X POST http://localhost:3000/convert \
  -F "file=@document.html" \
  -F "printBackground=true" \
  -o output.pdf
```

### Q: 如何在生产环境部署？

**A:** 推荐使用 Docker：

```bash
# 构建镜像
docker build -t pdfmill:production .

# 运行容器
docker run -d \
  --name pdfmill \
  -p 3000:3000 \
  --restart unless-stopped \
  pdfmill:production
```

### Q: 支持并发请求吗？

**A:** 是的！PDFMill 使用 Tokio 异步运行时，天然支持高并发。

## 下一步

- 查看完整文档: [README.md](README.md)
- 集成到你的应用中
- 根据需求调整配置
- 添加监控和日志

## 获取帮助

如果遇到问题：

1. 检查引擎是否已正确安装
2. 查看日志输出: `RUST_LOG=pdfmill=debug cargo run`
3. 运行测试脚本: `./test.sh`
4. 提交 Issue 到项目仓库

祝使用愉快！🚀
