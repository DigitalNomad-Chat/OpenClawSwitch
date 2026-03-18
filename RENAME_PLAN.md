# 项目重命名执行方案

## 📋 项目信息

**原名称**: Clawlite
**新名称**: Clawlite

**重命名范围**: 21 个文件，56 处引用

---

## 🎯 重命名规则

| 原名称格式 | 新名称格式 | 说明 |
|-----------|-----------|------|
| `Clawlite` | `Clawlite` | 产品名称（大驼峰） |
| `openclawswitch` | `clawlite` | 包名/标识符（全小写） |
| `open-claw-switch` | `clawlite` | 移除连字符，统一使用小写 |
| `Clawlite` | `Clawlite` | 文档标题 |
| `com.openclawswitch.app` | `com.clawlite.app` | Bundle ID |

---

## 📁 文件修改清单

### 1. 配置文件 (7 个)

| 文件 | 修改项 | 优先级 |
|------|--------|--------|
| `package.json` | name, description | ⭐⭐⭐ 高 |
| `src-tauri/tauri.conf.json` | productName, identifier, title | ⭐⭐⭐ 高 |
| `src-tauri/Cargo.toml` | name, description, repository | ⭐⭐⭐ 高 |
| `index.html` | title | ⭐⭐⭐ 高 |
| `.github/workflows/release.yml` | workflow 配置 | ⭐⭐ 中 |
| `build.bat` | 脚本注释 | ⭐ 低 |
| `build.sh` | 脚本注释 | ⭐ 低 |

### 2. 源代码文件 (5 个)

| 文件 | 修改项 | 优先级 |
|------|--------|--------|
| `src/App.vue` | 应用名称显示 | ⭐⭐⭐ 高 |
| `src/components/dashboard/TopBar.vue` | 标题显示 | ⭐⭐⭐ 高 |
| `src/components/pages/MessageChannelsPage.vue` | 注释 | ⭐ 低 |
| `src-tauri/src/installer.rs` | 错误消息 | ⭐⭐ 中 |
| `src-tauri/src/ssh.rs` | 注释 | ⭐ 低 |

### 3. Domain 文件 (3 个)

| 文件 | 修改项 | 优先级 |
|------|--------|--------|
| `src/domain/postInstallError.ts` | 错误消息 | ⭐⭐ 中 |
| `src/domain/postInstallError.test.ts` | 测试内容 | ⭐ 低 |
| `src/domain/quickSetupSession.ts` | 存储 key | ⭐⭐ 中 |

### 4. 文档文件 (4 个)

| 文件 | 修改项 | 优先级 |
|------|--------|--------|
| `README.md` | 所有引用 | ⭐⭐⭐ 高 |
| `README_EN.md` | 所有引用 | ⭐⭐⭐ 高 |
| `LICENSE` | 版权声明 | ⭐ 低 |
| `COLOR_AUDIT_REPORT.md` | 报告内容 | ⭐ 低 |
| `COLOR_UPDATE_REPORT.md` | 报告内容 | ⭐ 低 |

### 5. 脚本文件 (1 个)

| 文件 | 修改项 | 优先级 |
|------|--------|--------|
| `scripts/build-mac-universal.sh` | 脚本注释 | ⭐ 低 |

---

## 🔧 执行步骤

### 步骤 1: 修改核心配置文件（高优先级）

**顺序**:
1. `package.json` - npm 包名
2. `src-tauri/Cargo.toml` - Rust 包名
3. `src-tauri/tauri.conf.json` - 应用配置
4. `index.html` - 页面标题

### 步骤 2: 修改源代码文件

1. `src/App.vue`
2. `src/components/dashboard/TopBar.vue`
3. `src-tauri/src/installer.rs`
4. `src-tauri/src/ssh.rs`
5. `src/components/pages/MessageChannelsPage.vue`
6. `src/domain/postInstallError.ts`
7. `src/domain/postInstallError.test.ts`
8. `src/domain/quickSetupSession.ts`

### 步骤 3: 修改文档和脚本

1. `README.md` 和 `README_EN.md`
2. `.github/workflows/release.yml`
3. `build.bat` 和 `build.sh`
4. `scripts/build-mac-universal.sh`
5. `LICENSE`
6. `COLOR_AUDIT_REPORT.md` 和 `COLOR_UPDATE_REPORT.md`

### 步骤 4: 验证和测试

1. TypeScript 编译检查
2. Rust 编译检查
3. 应用启动测试
4. 功能验证

---

## ⚠️ 注意事项

### 需要特别小心的地方：

1. **Bundle ID**: `com.openclawswitch.app` → `com.clawlite.app`
   - 这会影响应用签名和更新机制

2. **存储 key**: `quickSetupSession.ts` 中的 key
   - 可能影响用户设置的迁移

3. **错误消息**: `postInstallError.ts` 和 `installer.rs`
   - 确保用户看到的错误信息正确

4. **文档一致性**: README 和注释
   - 保持新旧版本描述一致

### 需要手动验证的地方：

1. GitHub 仓库名称（需要手动在 GitHub 上修改）
2. 应用图标（如果有文字）
3. 安装包名称
4. 路径配置

---

## 📊 预期结果

修改完成后：
- ✅ 所有配置文件使用新名称
- ✅ 应用窗口标题显示 "Clawlite"
- ✅ 包名从 `openclawswitch` 变为 `clawlite`
- ✅ Bundle ID 从 `com.openclawswitch.app` 变为 `com.clawlite.app`
- ✅ 文档和注释统一使用新名称

---

## 🚀 开始执行

准备就绪，开始按步骤执行重命名。
