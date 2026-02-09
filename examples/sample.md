# PDFMill 示例文档

这是一个 Markdown 格式的测试文件。

## 主要特性

- 智能路由系统
- 多引擎支持
- 高性能异步处理
- 简单易用的 API

## 代码示例

使用 curl 转换文件:

```bash
curl -X POST http://localhost:3000/convert \
  -F "file=@sample.md" \
  -o output.pdf
```

## 支持的格式

### Web 格式
- HTML
- XHTML  
- Markdown

### Office 格式
- Word (.doc, .docx)
- Excel (.xls, .xlsx)
- PowerPoint (.ppt, .pptx)

### 图片格式
- JPG / JPEG
- PNG
- GIF
- BMP

---

**生成于**: 2026-01-27  
**版本**: PDFMill v0.1.0
