# 内容审核员系统状态

## 系统状态概述

本文档定义了内容审核员Agent的系统状态监控接口和状态标准。系统状态监控是确保审核服务稳定运行的重要保障，通过实时监控关键指标，及时发现和处理系统异常，保证审核服务的可用性和可靠性。

## 状态检查接口

### 健康检查接口

**接口地址**：/api/health

**检查方式**：HTTP GET请求

**响应格式**：

```json
{
  "status": "healthy|degraded|unhealthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0",
  "uptime": 86400,
  "components": {
    "text_analyzer": "operational",
    "image_analyzer": "operational",
    "video_analyzer": "operational",
    "stream_monitor": "operational",
    "knowledge_base": "operational"
  }
}
```

**状态定义**：

- healthy：所有组件正常运行
- degraded：部分组件降级运行，但核心功能可用
- unhealthy：核心组件不可用，需要立即处理

### 性能指标接口

**接口地址**：/api/metrics

**检查方式**：HTTP GET请求

**响应格式**：

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "performance": {
    "avg_response_time_ms": 120,
    "max_response_time_ms": 350,
    "throughput_per_second": 1500,
    "queue_depth": 45
  },
  "accuracy": {
    "auto_review_rate": 0.95,
    "false_positive_rate": 0.02,
    "false_negative_rate": 0.01
  },
  "availability": {
    "uptime_percentage": 99.95,
    "total_downtime_minutes": 5
  }
}
```

## 告警阈值

### 性能告警

- 响应时间超过500ms：warning级别
- 响应时间超过1000ms：critical级别
- 吞吐量低于正常值的50%：warning级别
- 队列积压超过1000条：warning级别
- 队列积压超过5000条：critical级别

### 可用性告警

- 系统不可用超过1分钟：critical级别
- 错误率超过5%：warning级别
- 错误率超过10%：critical级别

### 准确性告警

- 误报率（false positive）超过5%：warning级别
- 漏报率（false negative）超过2%：warning级别

## 状态上报配置

系统状态每60秒自动上报一次，当状态发生异常变化时立即触发告警通知。

## 依赖服务状态

### 外部依赖

| 服务 | 检查方式 | 告警阈值 |
|------|---------|---------|
| 网络连接 | Ping检测 | 连续3次失败 |
| 数据库连接 | 连接测试 | 连接超时10秒 |
| 缓存服务 | 读写测试 | 响应超时5秒 |
| 文件存储 | 读写测试 | 响应超时10秒 |

### 内部组件

| 组件 | 状态 | 最后检查时间 |
|------|------|-------------|
| 文本分析引擎 | operational | 2024-01-15T10:30:00Z |
| 图像识别引擎 | operational | 2024-01-15T10:30:00Z |
| 视频处理服务 | operational | 2024-01-15T10:30:00Z |
| 直播流接入器 | operational | 2024-01-15T10:30:00Z |
| 知识库 | operational | 2024-01-15T10:30:00Z |
| 报告生成器 | operational | 2024-01-15T10:30:00Z |
