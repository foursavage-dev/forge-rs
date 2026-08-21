# Docker 容器后端

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](../TRANSLATION.md)。

Docker 后端支持基于 `Dockerfile` 进行容器镜像的构建编排与缓存优化。

## 自动检测与任务
- **检测**: 存在 `Dockerfile`
- **构建**: `docker build -t <image-tag> .`
