# Sprint 1：核心页面迁移执行计划

> **目标：** 打通 remote/manual_mount 模式下的数据流，让三大核心配置页面真正读写远程配置。  
> **状态：** 计划中，待用户指令后执行  
> **预计工期：** 1 ~ 1.5 个工作日  
> **风险等级：** 高（涉及 App.vue 核心状态逻辑，需确保本地模式零回归）

---

## 一、前置条件

1. `useWorkspaceConfig.ts` 已就绪，提供 `loadConfig()` / `saveConfig()` / `checkEnvironment()` 等方法。
2. `config_resolver.rs` 已就绪，`workspace_read_config` / `workspace_write_config` 在 `workspace_id=null` 时与本地命令等价。
3. `npm run build` 和 Rust 编译当前均可通过。

---

## 二、本次 Sprint 范围

### 必须完成
- 迁移 `ConfigPage.vue`（AI 模型配置）到 `useWorkspaceConfig`
- 迁移 `BindingsPage.vue`（绑定管理）到 `useWorkspaceConfig`
- 迁移 `MessageChannelsPage.vue`（消息渠道）到 `useWorkspaceConfig`
- 迁移 `App.vue` 核心状态同步逻辑（`syncConfigSignals`、`checkEnvironment`、`resolveCurrentConfigSource`）到 Workspace-aware

### 明确不做
- 不新增 UI 组件（除必要的错误提示调整外）
- 不改动 Rust 后端（已有命令足够）
- 不处理 `auto_mount` 生命周期联动（放到 Sprint 2）
- 不删除旧 `currentEnv` 模型本身（仅在本 Sprint 内让旧逻辑与新逻辑桥接，彻底清理放到 Sprint 3）

---

## 三、任务拆分与详细执行步骤

### 任务 1：迁移 `ConfigPage.vue`

**当前问题：**
- 第 157 行直接调用 `load_default_config`
- 第 230 行直接调用 `save_config`
- 第 257 行直接调用 `save_config_as`
- 第 929、961、987、1060 行有旧 SSH 分支（`ssh_search_config`、`ssh_read_file`、`ssh_write_file`）
- 组件 props 中声明了 `envMode` 和 `envSshConnected`，页面内有多处 `if (props.envMode === 'ssh')` 分支

**执行步骤：**

1. **删除旧 props**
   - 移除 `defineProps` 中的 `envMode` 和 `envSshConnected`
   - 移除 `import type { ConfigFileInfo } from '@/types/config'`（若不再直接使用）

2. **引入 `useWorkspaceConfig`**
   - 在 `<script setup>` 顶部引入 `useWorkspaceConfig`
   - 实例化：`const { configSource, loadConfig, saveConfig, loading, saving, error } = useWorkspaceConfig()`

3. **替换配置加载逻辑**
   - 找到现有的 `loadConfig()` 函数（页面内可能有自定义命名，注意避免冲突，建议将页面内原有 `loadConfig` 重命名为 `loadPageConfig`）
   - 将 `invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')` 替换为 `useWorkspaceConfig().loadConfig()`
   - 加载成功后从 `configSource.value.config` 获取配置对象

4. **替换配置保存逻辑**
   - 将所有 `invoke('save_config', ...)` 替换为 `useWorkspaceConfig().saveConfig(updatedConfig)`
   - 将 `invoke('save_config_as', ...)` 替换为同样的 `saveConfig(updatedConfig)`（因为 `workspace_write_config` 内部已处理路径解析，无需 `save_as` 的显式路径参数）

5. **删除旧 SSH 分支**
   - 搜索并删除页面内所有 `ssh_search_config`、`ssh_read_file`、`ssh_write_file` 调用
   - 将原来用于远程文件浏览器或远程路径选择的逻辑，改为直接使用 `configSource.value.fileInfo.path`

6. **调整加载状态**
   - 将页面内自定义的 `loading` 状态与 `useWorkspaceConfig().loading` 合并或替换
   - 确保保存按钮的禁用状态绑定到 `saving`

7. **调整错误处理**
   - 保存失败时读取 `useWorkspaceConfig().error` 或通过 `try/catch` 捕获

**验收标准：**
- [ ] `npm run build` 无报错
- [ ] 本地模式下，ConfigPage 的加载/保存行为与迁移前完全一致
- [ ] manual_mount 模式下，修改模型配置后，挂载路径下的 `openclaw.json` 内容同步变更
- [ ] ssh_channel 模式下，修改模型配置后，远程 `~/.openclaw/openclaw.json` 内容同步变更

---

### 任务 2：迁移 `BindingsPage.vue`

**当前问题：**
- 第 319 行直接调用 `load_default_config`
- 第 521 行直接调用 `save_config`
- 组件 props 中声明了 `envMode` 和 `envSshConnected`

**执行步骤：**

1. **删除旧 props**
   - 移除 `defineProps` 中的 `envMode` 和 `envSshConnected`

2. **引入 `useWorkspaceConfig`**
   - 实例化 `useWorkspaceConfig()`

3. **替换加载逻辑**
   - 将 `load_default_config` 替换为 `useWorkspaceConfig().loadConfig()`
   - 绑定解析用 `parse_bindings` 时传入 `configSource.value.config`

4. **替换保存逻辑**
   - 将 `save_config` 替换为 `useWorkspaceConfig().saveConfig(config)`

5. **删除 SSH 分支**
   - 检查页面内是否有 `envMode === 'ssh'` 分支，如有则删除并统一走新逻辑

**验收标准：**
- [ ] `npm run build` 无报错
- [ ] 本地/remote 模式下绑定增删改查正常

---

### 任务 3：迁移 `MessageChannelsPage.vue`

**当前问题：**
- 第 1286 行有 `loadLocalConfig` 调用 `load_default_config`
- 第 2020 行有 `save_config` 调用
- 页面内可能无 `envMode` props，但保存逻辑直接走本地命令

**执行步骤：**

1. **引入 `useWorkspaceConfig`**
   - 实例化 `useWorkspaceConfig()`

2. **替换加载逻辑**
   - 将 `loadLocalConfig()` 中的 `load_default_config` 替换为 `useWorkspaceConfig().loadConfig()`

3. **替换保存逻辑**
   - 将 `save_config` 替换为 `useWorkspaceConfig().saveConfig(config)`

4. **状态同步**
   - 确保保存后重新加载页面数据时调用的是新的 `loadConfig()`

**验收标准：**
- [ ] `npm run build` 无报错
- [ ] 本地/remote 模式下消息渠道配置保存正常

---

### 任务 4：迁移 `App.vue` 核心状态同步逻辑

**当前问题：**
- `syncConfigSignals`（第 546 行）中：本地走 `load_default_config`，SSH 走 `ssh_search_config` + `ssh_read_file`
- `checkEnvironment`（第 768 行）中：本地走 `check_environment`，SSH 走 `ssh_check_environment`
- `resolveCurrentConfigSource`（第 1121 行）中：本地走 `load_default_config`，SSH 走 `ssh_search_config` + `ssh_read_file`
- 多处使用 `currentEnv.value.mode === 'ssh'` 判断，没有识别 `activeWorkspaceId`

**执行步骤：**

1. **引入 `useWorkspaceConfig`**
   - 在 `useWorkspaceStore` 引入处旁边引入 `useWorkspaceConfig`
   - 实例化：`const workspaceConfig = useWorkspaceConfig()`

2. **重构 `syncConfigSignals`**
   - 将原有的 `if (currentEnv.value.mode === 'ssh' && sshConnected.value)` 分支替换为：
     ```ts
     if (!workspaceStore.isLocalMode.value) {
       // 远程模式：统一通过 workspace_read_config 加载
       const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('workspace_read_config', {
         workspaceId: workspaceStore.activeWorkspaceId.value,
       })
       // ...后续解析逻辑与本地模式保持一致
     }
     ```
   - 本地模式（`workspaceStore.isLocalMode.value === true`）保持调用 `load_default_config`，确保零回归
   - 删除 `ssh_search_config` 和 `ssh_read_file` 的调用

3. **重构 `checkEnvironment`**
   - 将原有的 `if (currentEnv.value.mode === 'ssh')` 分支替换为：
     ```ts
     if (!workspaceStore.isLocalMode.value) {
       envStatus.value = await invoke<EnvironmentStatus>('workspace_check_environment', {
         workspaceId: workspaceStore.activeWorkspaceId.value,
       })
     } else {
       envStatus.value = await invoke<EnvironmentStatus>('check_environment')
     }
     ```
   - 删除对 `sshConnected.value` 的硬依赖（`workspace_check_environment` 内部会处理 SSH 连接状态）

4. **重构 `resolveCurrentConfigSource`**
   - 将本地/SSH 两套分支统一为：
     ```ts
     const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('workspace_read_config', {
       workspaceId: workspaceStore.activeWorkspaceId.value,
     })
     return { mode: workspaceStore.isLocalMode.value ? 'local' : 'ssh', path: info.path, config }
     ```
   - 注意：这里 `mode` 字段用于浏览器设置页的路径显示和保存路由，`'ssh'` 可以泛化为"非本地"，不影响现有逻辑
   - 删除 `ssh_search_config` 和 `ssh_read_file` 调用

5. **调整 `checkGatewayHealth` 和 `runGatewayStartCommand`**
   - `checkGatewayHealth`：非本地模式统一走 `workspace_health_check`
   - `runGatewayStartCommand`：非本地模式统一走 `workspace_restart_gateway` 或 SSH 相关的网关启动命令（视已有命令而定）

6. **调整 `handleWorkspaceChanged` 桥接逻辑**
   - 确保切换 Workspace 后自动触发 `checkEnvironment()`，而不是依赖旧的 `selectEnvironment()` 间接触发

**验收标准：**
- [ ] `npm run build` 无报错
- [ ] 本地模式下 Dashboard 统计数据、门禁状态、网关健康检查与迁移前一致
- [ ] manual_mount 模式下 Dashboard 读取的是挂载路径下的配置统计
- [ ] ssh_channel 模式下 Dashboard 读取的是远程配置统计

---

## 四、执行顺序与依赖关系

```
┌─────────────────────────────────────────────────────────────┐
│  任务 1：迁移 ConfigPage.vue                                  │
│  ───────────────────────────                                │
│  （独立任务，不依赖其他任务）                                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  任务 2：迁移 BindingsPage.vue                                │
│  ───────────────────────────                                │
│  （独立任务，与任务 1 可并行思考，但建议串行降低风险）         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  任务 3：迁移 MessageChannelsPage.vue                         │
│  ─────────────────────────────────                          │
│  （逻辑最简单，建议放在任务 2 之后）                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  任务 4：迁移 App.vue 核心逻辑                                │
│  ───────────────────────────                                │
│  （必须在任务 1~3 完成后执行，否则 Dashboard 与页面状态分裂）  │
└─────────────────────────────────────────────────────────────┘
```

**推荐执行顺序：** 1 → 2 → 3 → 4

**原因：**
- ConfigPage 最复杂，先做可以暴露最多的边界 case
- BindingsPage / MessageChannelsPage 相对简单，可巩固迁移模式
- App.vue 最后做，因为 Dashboard 的 `syncConfigSignals` 依赖正确的配置读取入口，必须等页面层迁移完毕后再统一收口

---

## 五、风险与应对策略

| 风险 | 影响 | 应对策略 |
|------|------|---------|
| `ConfigPage.vue` 逻辑复杂，直接替换 `loadConfig` 可能破坏局部状态 | 高 | 保留原函数签名，内部逐步替换；每一步修改后立即 `npm run build` 验证 |
| `App.vue` 的 `syncConfigSignals` 涉及大量统计指标解析 | 高 | 仅改变"配置来源获取方式"，不改变"解析统计指标"的逻辑；本地分支保持原样 |
| `saveConfig` 与 `save_config_as` 行为差异 | 中 | `workspace_write_config` 对于本地模式可以额外接受 `path` 参数（若当前不支持，则先扩展 Rust 命令），或直接废弃 `save_as` 功能（评估使用频率） |
| 旧 `envMode` / `envSshConnected` props 被其他父组件传递 | 中 | 在页面组件中直接删除 props，观察 IDE / 编译报错，同步清理父组件（如 App.vue）中的 prop 绑定 |
| `useWorkspaceConfig` 中 `configSource` 命名与页面内状态冲突 | 低 | 页面内解构时重命名：`const { configSource: wsConfig, loadConfig: loadWsConfig } = useWorkspaceConfig()` |

---

## 六、回滚策略

1. 执行前确保当前 `git status` 干净（当前已满足）。
2. 每个任务完成后执行 `git add -p` + `git commit`，不要一次性提交所有修改。
3. 若某个任务导致严重回归，使用 `git revert HEAD` 快速回滚到上一个稳定提交。

---

## 七、验收清单（Sprint 1 结束标准）

- [ ] `npm run build` 前端编译通过
- [ ] `cargo build` / `cargo check` Rust 编译通过
- [ ] 本地模式下，ConfigPage / BindingsPage / MessageChannelsPage 功能与迁移前完全一致
- [ ] manual_mount 模式下：
  - [ ] ConfigPage 修改并保存后，挂载路径下的 `openclaw.json` 内容变更
  - [ ] BindingsPage 增删改绑定后，挂载路径下的 `openclaw.json` 内容变更
  - [ ] MessageChannelsPage 修改渠道后，挂载路径下的 `openclaw.json` 内容变更
  - [ ] Dashboard 首页显示的模型状态、绑定数、渠道数与远程配置一致
- [ ] ssh_channel 模式下：
  - [ ] 上述三大页面通过 SSH 通道正确读写远程 `~/.openclaw/openclaw.json`
  - [ ] Dashboard 统计数据与远程配置一致
