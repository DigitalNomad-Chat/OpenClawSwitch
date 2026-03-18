
<template>
  <div class="oc-app-shell dashboard-shell">
    <div class="dashboard-container">
      <!-- 顶部命令栏 -->
      <TopBar
        :active-nav="activeNav"
        :theme-mode="themeMode"
        @navigate="navigateTo"
        @theme-change="applyThemeMode"
      />

      <!-- 主内容区域 -->
      <div class="dashboard-main">
        <!-- 仪表盘矩阵（正常状态） -->
        <DashboardGrid
          v-if="!isGateActive && !quickSetupForcedOpen"
          :active-nav="activeNav"
          @navigate="navigateTo"
        />

        <!-- 门禁状态内容 -->
        <div v-else-if="isGateActive || quickSetupForcedOpen" class="dashboard-content">
          <div class="oc-main-scroll-page">
            <div
              v-if="isGateActive && (gateState === 'NO_TARGET' || (gateState === 'NEED_INSTALL' && targetMode === 'ssh') || (gateState === 'NEED_CONFIG' && targetMode === 'ssh'))"
              class="oc-panel p-6"
            >
              <h3 class="text-xl font-semibold" style="color: var(--oc-text-primary);">
                {{
                  gateState === 'NO_TARGET'
                    ? '选择运行环境'
                    : gateState === 'NEED_INSTALL'
                      ? '完成安装'
                      : '完成模型配置'
                }}
              </h3>
              <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">
                {{
                  gateState === 'NO_TARGET'
                    ? '请选择本地或 SSH 作为唯一运行环境入口。'
                    : gateState === 'NEED_INSTALL'
                      ? '当前环境已确定，请继续完成安装。'
                      : '环境已安装，下一步是完成配置并验证生效。'
                }}
              </p>

              <div v-if="gateState === 'NO_TARGET'" class="mt-4 grid gap-3 md:grid-cols-2">
                <button
                  type="button"
                  class="rounded-[12px] border p-4 text-left transition-colors"
                  :style="{
                    borderColor: targetMode === 'local' ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
                    background: targetMode === 'local' ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)'
                  }"
                  @click="chooseLocalTarget"
                >
                  <p class="text-base font-semibold" style="color: var(--oc-text-primary);">安装在本地</p>
                  <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">使用本机环境安装并运行 OpenClaw。</p>
                </button>

                <button
                  type="button"
                  class="rounded-[12px] border p-4 text-left transition-colors"
                  :style="{
                    borderColor: targetMode === 'ssh' ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
                    background: targetMode === 'ssh' ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)'
                  }"
                  @click="chooseSshTarget"
                >
                  <p class="text-base font-semibold" style="color: var(--oc-text-primary);">连接 SSH 环境</p>
                  <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">后续所有操作基于该 SSH 环境执行。</p>
                </button>
              </div>

              <div v-else-if="gateState === 'NEED_INSTALL' && targetMode === 'ssh'" class="mt-4 space-y-3">
                <div class="rounded-[12px] border px-3 py-2 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);">
                  当前环境：<strong style="color: var(--oc-text-primary);">{{ targetMode === 'ssh' ? 'SSH' : '本地' }}</strong>
                </div>

                <div v-if="targetMode === 'ssh'" class="rounded-[12px] border px-3 py-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);">
                  <p>1. 先通过右上角环境入口建立 SSH 连接</p>
                  <p class="mt-1">2. 在远程执行安装后，点击"重新检测环境"</p>
                  <div class="mt-3 flex flex-wrap gap-2">
                    <button class="oc-toolbar-btn h-9 px-3" type="button" @click="showSshModal = true">
                      连接 SSH
                    </button>
                    <button class="oc-toolbar-btn h-9 px-3" type="button" @click="checkEnvironment">
                      重新检测环境
                    </button>
                  </div>
                </div>

                <div class="flex flex-wrap gap-2">
                  <button class="oc-toolbar-btn h-9 px-3" type="button" @click="chooseLocalTarget">改用本地</button>
                  <button class="oc-toolbar-btn h-9 px-3" type="button" @click="chooseSshTarget">改用 SSH</button>
                </div>
              </div>

              <div v-else class="mt-4 space-y-2">
                <div class="rounded-[12px] border px-3 py-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);">
                  检测结果为"待配置"，请先完成以下任一操作：
                </div>
                <div class="flex flex-wrap gap-2">
                  <button class="oc-toolbar-btn h-10 px-4" type="button" @click="runManualConfig">
                    手动配置
                  </button>
                  <button class="oc-toolbar-btn h-10 px-4" type="button" @click="applyDefaultConfig">
                    使用默认配置
                  </button>
                  <button class="oc-toolbar-btn h-10 px-4" type="button" @click="checkEnvironment">
                    重新检测环境
                  </button>
                </div>
              </div>
            </div>

            <QuickSetupGuide
              v-else-if="shouldShowQuickSetupGuide && envStatus"
              class="h-full"
              :show-toast="showToast"
              :show-close-action="shouldShowQuickSetupCloseAction"
              :system-os="envStatus.system.os"
              @close="handleQuickSetupClose"
              @complete="handleQuickSetupComplete"
            />

            <InstallPage
              v-else-if="gateState === 'NEED_INSTALL' && targetMode === 'local'"
              class="h-full"
              :mode="'local'"
              :env-connected="true"
              :openclaw-installed="openclawInstalled"
              @install-complete="handleInstallComplete"
            />
          </div>
        </div>

        <!-- 正常页面导航内容 -->
        <div v-else class="dashboard-content">
          <div class="oc-main-scroll-page">
            <StatusDashboard
              v-if="activeNav === 'overview' && envStatus"
              class="oc-page-root"
              :env-status="envStatus"
              :gateway-reachable="gatewayReachable"
              :env-mode="currentEnv.mode"
              :is-windows="isWindows"
              :gateway-service-installed="gatewayServiceInstalled"
              :pending-tool-id="pendingToolId"
              @open-tool="openToolPanel"
            />

            <div v-else-if="activeNav === 'ai-config'" class="oc-page-root">
              <ConfigPage
                v-if="envStatus && openclawInstalled"
                class="oc-page-root"
                :show-toast="showToast"
                :env-mode="currentEnv.mode"
                :env-ssh-connected="sshConnected"
              />
              <div v-else class="oc-panel p-6">
                <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">模型配置不可用</h3>
                <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">请先在"安装与接入"中完成安装。</p>
              </div>
            </div>

            <div v-else-if="activeNav === 'bindings'" class="oc-page-root">
              <BindingsPage
                v-if="envStatus && openclawInstalled"
                class="oc-page-root"
                :show-toast="showToast"
                :env-mode="currentEnv.mode"
                :env-ssh-connected="sshConnected"
              />
              <div v-else class="oc-panel p-6">
                <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">绑定管理不可用</h3>
                <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">请先在"安装与接入"中完成安装。</p>
              </div>
            </div>

            <DiagnosticsPage
              v-else-if="activeNav === 'diagnostics'"
              class="oc-page-root"
              :app-state="appState === 'ERROR' || appState === 'DEGRADED' ? appState : 'READY'"
              :env-mode="currentEnv.mode"
              :openclaw-installed="openclawInstalled"
              @refresh="checkEnvironment"
            />

            <div v-else-if="activeNav === 'channels'" class="oc-page-root">
              <MessageChannelsPage class="h-full min-h-0" :show-toast="showToast" :system-os="currentSystemOs" />
            </div>

            <div v-else class="space-y-3">
              <section class="oc-panel p-6">
                <h3 class="text-xl font-semibold" style="color: var(--oc-text-primary);">系统设置</h3>
                <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">连接相关操作统一从顶部环境入口管理，设置页仅保留偏好项。</p>
              </section>

              <section class="oc-panel p-6">
                <div class="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <h4 class="text-lg font-semibold" style="color: var(--oc-text-primary);">工具设置</h4>
                    <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">
                      按当前环境修改配置文件，不覆盖你已有的其它字段。
                    </p>
                  </div>
                  <span class="rounded-[10px] border px-2.5 py-1 text-xs" style="border-color: var(--oc-card-border); color: var(--oc-text-secondary);">
                    {{ currentEnv.mode === 'ssh' ? 'SSH 环境' : '本地环境' }}
                  </span>
                </div>

                <div class="mt-4 rounded-[12px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                  <div class="flex flex-wrap items-center justify-between gap-3">
                    <div>
                      <p class="text-sm font-medium" style="color: var(--oc-text-primary);">浏览器默认 Profile</p>
                      <p class="mt-1 text-xs" style="color: var(--oc-text-muted);">
                        开启时写入 <code>browser.defaultProfile</code> = <code>"openclaw"</code>；关闭时仅删除 <code>defaultProfile</code>，保留 <code>browser</code> 及其它设置。
                      </p>
                    </div>
                    <div class="inline-flex items-center gap-3">
                      <button
                        type="button"
                        aria-label="toggle-browser-default-profile"
                        class="relative inline-flex h-6 w-11 items-center rounded-full border transition-colors"
                        :style="{
                          borderColor: browserDefaultProfileEnabled ? 'color-mix(in srgb, var(--oc-success) 55%, transparent)' : 'var(--oc-card-border)',
                          background: browserDefaultProfileEnabled
                            ? 'color-mix(in srgb, var(--oc-success) 28%, transparent)'
                            : 'color-mix(in srgb, var(--oc-card-elevated) 92%, transparent)'
                        }"
                        :disabled="browserSettingSwitchDisabled"
                        @click="toggleBrowserDefaultProfile"
                      >
                        <span
                          class="h-4 w-4 rounded-full border transition-transform"
                          :style="{
                            borderColor: 'var(--oc-card-border)',
                            background: 'var(--oc-card)',
                            transform: browserDefaultProfileEnabled ? 'translateX(22px)' : 'translateX(2px)'
                          }"
                        />
                      </button>
                      <span class="text-xs" :style="{ color: browserDefaultProfileEnabled ? 'var(--oc-success)' : 'var(--oc-text-muted)' }">
                        {{ browserSettingStatusText }}
                      </span>
                    </div>
                  </div>
                </div>

                <p v-if="browserSettingPath" class="mt-3 text-xs" style="color: var(--oc-text-muted);">
                  配置文件：{{ browserSettingPath }}
                </p>
                <p v-if="browserSettingError" class="mt-2 text-xs" style="color: var(--oc-danger);">
                  {{ browserSettingError }}
                </p>
              </section>

              <section class="oc-panel p-6">
                <div class="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <h4 class="text-lg font-semibold" style="color: var(--oc-text-primary);">页面调试</h4>
                    <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">
                      从设置页直接打开快速引导，便于调试布局、主题色和页面内容。
                    </p>
                  </div>
                  <span
                    class="rounded-[10px] border px-2.5 py-1 text-xs"
                    style="border-color: color-mix(in srgb, var(--oc-accent) 12%, var(--oc-card-border)); color: var(--oc-accent);"
                  >
                    调试入口
                  </span>
                </div>

                <div
                  class="mt-4 rounded-[12px] border p-4"
                  style="border-color: color-mix(in srgb, var(--oc-accent) 10%, var(--oc-card-border)); background: color-mix(in srgb, var(--oc-accent-soft) 18%, var(--oc-card) 82%);"
                >
                  <div class="flex flex-wrap items-center justify-between gap-4">
                    <div>
                      <p class="text-sm font-medium" style="color: var(--oc-text-primary);">打开快速引导页面</p>
                      <p class="mt-1 text-xs leading-6" style="color: var(--oc-text-secondary);">
                        使用当前本地环境数据渲染快速引导，用于检查满高布局与细节视觉效果。
                      </p>
                    </div>

                    <Button variant="default" @click="openQuickSetupDebug">
                      打开快速引导
                    </Button>
                  </div>
                </div>
              </section>

              <section v-if="showOpenClawUninstallAction" class="oc-panel p-6">
                <div class="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <h4 class="text-lg font-semibold" style="color: var(--oc-text-primary);">危险操作</h4>
                    <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">
                      卸载本机 OpenClaw 全局 npm 包，并移除后台网关服务。
                    </p>
                  </div>
                  <span
                    class="rounded-[10px] border px-2.5 py-1 text-xs"
                    style="border-color: color-mix(in srgb, var(--oc-danger) 24%, var(--oc-card-border)); color: var(--oc-danger);"
                  >
                    仅本地环境
                  </span>
                </div>

                <div
                  class="mt-4 rounded-[12px] border p-4"
                  style="border-color: color-mix(in srgb, var(--oc-danger) 24%, var(--oc-card-border)); background: color-mix(in srgb, var(--oc-danger) 6%, var(--oc-card));"
                >
                  <div class="flex flex-wrap items-start justify-between gap-4">
                    <div class="max-w-2xl">
                      <p class="text-sm font-medium" style="color: var(--oc-text-primary);">卸载 OpenClaw</p>
                      <p class="mt-1 text-xs leading-6" style="color: var(--oc-text-muted);">
                        会删除全局 <code>openclaw</code> npm 包并卸载网关后台服务。Windows 下也会尝试卸载通过
                        <code>nssm</code> 安装的 <code>openclaw-gateway</code> 服务；最后一步可选择是否删除
                        <code>~/.openclaw</code>。
                      </p>
                    </div>

                    <Button
                      variant="destructive"
                      :disabled="openClawUninstallActionState.disabled"
                      :title="openClawUninstallActionState.reason || '卸载 OpenClaw'"
                      @click="openOpenClawUninstallFlow"
                    >
                      卸载 OpenClaw
                    </Button>
                  </div>
                </div>

                <p
                  v-if="openClawUninstallActionState.reason"
                  class="mt-3 text-xs"
                  style="color: var(--oc-text-muted);"
                >
                  {{ openClawUninstallActionState.reason }}
                </p>
              </section>
            </div>
          </div>
        </div>
      </div>

      <!-- 底部 Dock 栏 -->
      <DockBar
        :active-nav="activeNav"
        @navigate="navigateTo"
      />
    </div>

    <!-- SSH 连接模态框 -->
    <SshConnectModal
      v-if="showSshModal"
      @close="showSshModal = false"
      @connected="handleSshConnected"
      @fingerprint="handleFingerprint"
    />

    <SshFingerprintDialog
      v-if="showFingerprintDialog && sshFingerprint"
      :fingerprint="sshFingerprint"
      @confirm="confirmFingerprint"
      @reject="rejectFingerprint"
    />

    <div
      v-if="uninstallOpenClawStep === 'confirm'"
      class="oc-modal-overlay"
      @click.self="closeOpenClawUninstallFlow"
    >
      <Card class="oc-modal-card w-full max-w-lg p-6">
        <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">确认卸载 OpenClaw</h3>
        <p class="mt-1 text-sm leading-6" style="color: var(--oc-text-muted);">
          卸载会按安装流程反向清理当前用户下的 OpenClaw 组件；本步骤默认保留 <code>~/.openclaw</code>，下一步可选是否一并删除配置目录。
        </p>

        <ul class="mt-4 space-y-2 text-sm leading-6" style="color: var(--oc-text-secondary);">
          <li v-for="item in uninstallCleanupItemsWithoutConfig" :key="item" class="flex gap-2">
            <span style="color: var(--oc-danger);">•</span>
            <span>{{ item }}</span>
          </li>
        </ul>

        <div class="mt-5 flex justify-end gap-2">
          <Button variant="outline" :disabled="uninstallOpenClawLoading" @click="closeOpenClawUninstallFlow">
            取消
          </Button>
          <Button variant="destructive" :disabled="uninstallOpenClawLoading" @click="continueOpenClawUninstallFlow">
            继续卸载
          </Button>
        </div>
      </Card>
    </div>

    <div
      v-if="uninstallOpenClawStep === 'phrase'"
      class="oc-modal-overlay"
      @click.self="closeOpenClawUninstallFlow"
    >
      <Card class="oc-modal-card w-full max-w-lg p-6">
        <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">输入确认短语</h3>
        <p class="mt-1 text-sm leading-6" style="color: var(--oc-text-muted);">
          请输入下面的确认短语后继续卸载。
        </p>

        <div class="mt-4 rounded-[14px] border p-3" style="border-color: color-mix(in srgb, var(--oc-danger) 28%, transparent); background: color-mix(in srgb, var(--oc-danger) 8%, transparent);">
          <div class="flex flex-wrap items-center gap-2">
            <button
              type="button"
              class="rounded-[10px] px-3 py-2 text-sm font-bold transition-opacity hover:opacity-85"
              style="background: color-mix(in srgb, var(--oc-danger) 14%, transparent); color: var(--oc-danger);"
              :disabled="uninstallOpenClawLoading"
              @click="copyOpenClawUninstallPhrase"
            >
              {{ OPENCLAW_UNINSTALL_CONFIRM_PHRASE }}
            </button>
            <Button variant="outline" size="sm" :disabled="uninstallOpenClawLoading" @click="copyOpenClawUninstallPhrase">
              点击复制
            </Button>
          </div>
        </div>

        <div class="mt-4">
          <Input
            :model-value="uninstallOpenClawInput"
            :placeholder="OPENCLAW_UNINSTALL_CONFIRM_PHRASE"
            :disabled="uninstallOpenClawLoading"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            @update:model-value="uninstallOpenClawInput = String($event)"
          />
        </div>

        <p
          class="mt-2 text-xs"
          :style="{ color: uninstallOpenClawInput && !uninstallOpenClawPhraseValid ? 'var(--oc-danger)' : 'var(--oc-text-quiet)' }"
        >
          {{
            uninstallOpenClawInput && !uninstallOpenClawPhraseValid
              ? '确认短语不匹配，请完整输入。'
              : `请完整输入：${OPENCLAW_UNINSTALL_CONFIRM_PHRASE}`
          }}
        </p>

        <div class="mt-5 flex justify-end gap-2">
          <Button variant="outline" :disabled="uninstallOpenClawLoading" @click="closeOpenClawUninstallFlow">
            取消
          </Button>
          <Button
            variant="destructive"
            :disabled="uninstallOpenClawLoading || !uninstallOpenClawPhraseValid"
            @click="confirmOpenClawUninstallPhraseStep"
          >
            继续
          </Button>
        </div>
      </Card>
    </div>

    <div
      v-if="uninstallOpenClawStep === 'config'"
      class="oc-modal-overlay"
      @click.self="closeOpenClawUninstallFlow"
    >
      <Card class="oc-modal-card w-full max-w-lg p-6">
        <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">是否删除 ~/.openclaw</h3>
        <p class="mt-1 text-sm leading-6" style="color: var(--oc-text-muted);">
          如果一并删除，将把本地配置、工作区、缓存、日志和托管运行时一起清理，同时回收为 OpenClaw 写入的用户环境配置。
        </p>

        <ul class="mt-4 space-y-2 text-sm leading-6" style="color: var(--oc-text-secondary);">
          <li v-for="item in uninstallCleanupItemsWithConfig" :key="item" class="flex gap-2">
            <span style="color: var(--oc-danger);">•</span>
            <span>{{ item }}</span>
          </li>
        </ul>

        <div class="mt-5 flex flex-wrap justify-end gap-2">
          <Button variant="outline" :disabled="uninstallOpenClawLoading" @click="closeOpenClawUninstallFlow">
            取消
          </Button>
          <Button variant="outline" :disabled="uninstallOpenClawLoading" @click="runOpenClawUninstall(false)">
            仅卸载，不删配置
          </Button>
          <Button variant="destructive" :disabled="uninstallOpenClawLoading" @click="runOpenClawUninstall(true)">
            删除配置并卸载
          </Button>
        </div>
      </Card>
    </div>
    <Toast v-if="toast" :type="toast.type" :message="toast.message" @close="closeToast" />

    <div v-if="loading" class="fixed inset-0 z-[110] flex items-center justify-center bg-black/30 backdrop-blur-[1px]">
      <div class="flex items-center gap-3 rounded-xl border px-4 py-3" style="border-color: var(--oc-card-border); background: var(--oc-card); box-shadow: var(--oc-shadow-popover);">
        <div class="h-5 w-5 animate-spin rounded-full border-2 border-[var(--oc-accent)] border-t-transparent" />
        <span class="text-sm" style="color: var(--oc-text-primary);">{{ loadingMessage }}</span>
      </div>
    </div>
  </div>
</template>
