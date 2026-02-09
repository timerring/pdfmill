# PDFMill Docker 部署指南

## 快速启动

### 方法 1: 使用 docker-compose (推荐)

```bash
# 构建并启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down

# 完全清理（包括数据卷）
docker-compose down -v
```

### 方法 2: 使用 Docker 命令

```bash
# 构建镜像
docker build -t pdfmill:latest .

# 运行容器
docker run -d \
  --name pdfmill \
  -p 3000:3000 \
  --restart unless-stopped \
  pdfmill:latest

# 查看日志
docker logs -f pdfmill

# 停止并删除容器
docker stop pdfmill && docker rm pdfmill
```

## 测试服务

```bash
# 检查服务状态
curl http://localhost:3000/health

# 查看服务信息
curl http://localhost:3000/info | jq

# 转换示例文件
curl -X POST http://localhost:3000/convert \
  -F "file=@examples/sample.html" \
  -o output.pdf
```

## 构建说明

### 构建时间

首次构建大约需要 **10-15 分钟**，主要时间花在：
- Rust 依赖编译：~5 分钟
- 系统依赖安装（Chromium, LibreOffice, ImageMagick）：~5 分钟

后续构建会利用 Docker 缓存，只需 **1-2 分钟**。

### 镜像大小

- **Builder stage**: ~2.5 GB（包含 Rust 编译器和工具链）
- **Runtime stage**: ~1.2 GB（只包含运行时依赖）
- **最终镜像**: ~1.2 GB

包含的组件：
- Debian Bookworm Slim 基础镜像
- Chromium 浏览器（~300 MB）
- LibreOffice（~400 MB）
- ImageMagick（~50 MB）
- 字体包（中文字体支持）
- PDFMill 二进制文件（~20 MB）

## 配置选项

### 环境变量

在 `docker-compose.yml` 或 `docker run` 命令中设置：

```yaml
environment:
  # 服务配置
  - PDFMILL_ADDR=0.0.0.0:3000
  - RUST_LOG=pdfmill=debug  # debug, info, warn, error
  
  # 引擎路径（通常不需要修改）
  - CHROME_PATH=/usr/bin/chromium
  - SOFFICE_PATH=/usr/bin/soffice
  - CONVERT_PATH=/usr/bin/convert
```

### 端口映射

默认映射到主机的 3000 端口，可以修改：

```bash
# 映射到其他端口
docker run -p 8080:3000 pdfmill:latest

# docker-compose.yml
ports:
  - "8080:3000"
```

### 资源限制

推荐设置资源限制防止过度消耗：

```yaml
# docker-compose.yml
deploy:
  resources:
    limits:
      cpus: '2'       # 最多使用 2 个 CPU
      memory: 2G      # 最多使用 2GB 内存
    reservations:
      cpus: '0.5'     # 最少保留 0.5 CPU
      memory: 512M    # 最少保留 512MB 内存
```

Docker CLI 方式：

```bash
docker run -d \
  --cpus="2" \
  --memory="2g" \
  --memory-reservation="512m" \
  -p 3000:3000 \
  pdfmill:latest
```

### 数据持久化

挂载临时文件目录（可选）：

```bash
docker run -d \
  -p 3000:3000 \
  -v pdfmill-tmp:/tmp/pdfmill \
  pdfmill:latest
```

## 生产环境部署

### 1. 使用 Docker Swarm

```bash
# 初始化 Swarm
docker swarm init

# 部署服务
docker stack deploy -c docker-compose.yml pdfmill

# 扩展到 3 个实例
docker service scale pdfmill_pdfmill=3

# 查看服务状态
docker service ls
docker service ps pdfmill_pdfmill
```

### 2. 使用 Kubernetes

创建 `k8s-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pdfmill
spec:
  replicas: 3
  selector:
    matchLabels:
      app: pdfmill
  template:
    metadata:
      labels:
        app: pdfmill
    spec:
      containers:
      - name: pdfmill
        image: pdfmill:latest
        ports:
        - containerPort: 3000
        env:
        - name: RUST_LOG
          value: "pdfmill=info"
        resources:
          limits:
            cpu: "2"
            memory: "2Gi"
          requests:
            cpu: "500m"
            memory: "512Mi"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 60
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: pdfmill
spec:
  selector:
    app: pdfmill
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
```

部署：

```bash
kubectl apply -f k8s-deployment.yaml
kubectl get pods
kubectl get svc pdfmill
```

### 3. 配合 Nginx 反向代理

`nginx.conf`:

```nginx
upstream pdfmill {
    server localhost:3000;
}

server {
    listen 80;
    server_name pdf.yourdomain.com;
    
    # 增加超时时间（PDF 转换可能需要较长时间）
    proxy_connect_timeout 600;
    proxy_send_timeout 600;
    proxy_read_timeout 600;
    
    # 增加客户端上传限制
    client_max_body_size 50M;
    
    location / {
        proxy_pass http://pdfmill;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 监控和维护

### 查看日志

```bash
# Docker Compose
docker-compose logs -f --tail=100

# Docker
docker logs -f pdfmill

# 只看错误日志
docker logs pdfmill 2>&1 | grep -i error
```

### 健康检查

```bash
# 手动检查
curl http://localhost:3000/health

# Docker 内置健康检查
docker inspect --format='{{.State.Health.Status}}' pdfmill
```

### 性能监控

```bash
# 查看容器资源使用
docker stats pdfmill

# 持续监控
docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}"
```

### 备份和恢复

```bash
# 导出镜像
docker save pdfmill:latest | gzip > pdfmill-backup.tar.gz

# 导入镜像
gunzip -c pdfmill-backup.tar.gz | docker load
```

## 故障排查

### 问题 1: 容器无法启动

```bash
# 查看详细日志
docker logs pdfmill

# 检查容器状态
docker inspect pdfmill

# 进入容器调试
docker exec -it pdfmill bash
```

### 问题 2: 转换失败

可能原因：
- 引擎未正确安装
- 权限问题
- 内存不足

解决方法：

```bash
# 进入容器检查引擎
docker exec -it pdfmill bash
chromium --version
soffice --version
convert --version

# 增加内存限制
docker update --memory 3g pdfmill
```

### 问题 3: 性能问题

```bash
# 查看资源使用
docker stats pdfmill

# 增加 CPU 和内存
docker update --cpus 4 --memory 4g pdfmill

# 扩展多个实例（docker-compose）
docker-compose up -d --scale pdfmill=3
```

### 问题 4: 中文字体显示问题

容器已包含 `fonts-noto-cjk` 字体包，如需额外字体：

```dockerfile
# 在 Dockerfile 中添加
RUN apt-get install -y fonts-wqy-microhei fonts-wqy-zenhei
```

或者挂载主机字体：

```bash
docker run -d \
  -p 3000:3000 \
  -v /usr/share/fonts:/usr/share/fonts:ro \
  pdfmill:latest
```

## 安全建议

### 1. 不使用 root 运行

镜像已配置为使用非特权用户 `pdfmill` (UID 1001)。

### 2. 只暴露必要端口

```bash
# 只在本地监听
docker run -d -p 127.0.0.1:3000:3000 pdfmill:latest
```

### 3. 定期更新镜像

```bash
# 重新构建获取最新安全更新
docker-compose build --no-cache
docker-compose up -d
```

### 4. 使用 secrets 管理敏感信息

```yaml
# docker-compose.yml
services:
  pdfmill:
    secrets:
      - api_key
secrets:
  api_key:
    file: ./secrets/api_key.txt
```

## 性能优化

### 1. 使用多阶段构建缓存

镜像已优化为多阶段构建，依赖层会被缓存。

### 2. 并行构建

```bash
# 使用 BuildKit 加速构建
DOCKER_BUILDKIT=1 docker build -t pdfmill:latest .
```

### 3. 使用 Docker 卷加速临时文件

```bash
docker run -d \
  -p 3000:3000 \
  -v pdfmill-tmp:/tmp/pdfmill:rw \
  --tmpfs /tmp:rw,noexec,nosuid,size=1g \
  pdfmill:latest
```

## 更新和升级

```bash
# 拉取最新代码
git pull

# 重新构建并启动
docker-compose build
docker-compose up -d

# 清理旧镜像
docker image prune -f
```

## 完整启动示例

生产环境推荐配置：

```bash
# 1. 创建 docker-compose.prod.yml
cat > docker-compose.prod.yml << 'EOF'
version: '3.8'

services:
  pdfmill:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: pdfmill-prod
    ports:
      - "127.0.0.1:3000:3000"
    environment:
      - RUST_LOG=pdfmill=info
      - PDFMILL_ADDR=0.0.0.0:3000
    restart: always
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 1G
    volumes:
      - pdfmill-tmp:/tmp/pdfmill
    logging:
      driver: "json-file"
      options:
        max-size: "50m"
        max-file: "5"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

volumes:
  pdfmill-tmp:
EOF

# 2. 启动服务
docker-compose -f docker-compose.prod.yml up -d

# 3. 查看状态
docker-compose -f docker-compose.prod.yml ps
docker-compose -f docker-compose.prod.yml logs -f
```

现在你的 PDFMill 服务已经通过 Docker 运行了！🚀
