import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import obfuscator from "vite-plugin-bundle-obfuscator";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    obfuscator({
      apply: 'build',
      enable: true,
      autoExcludeNodeModules: true,
      threadPool: true,
      options: {
        compact: true,
        simplify: true,
        controlFlowFlattening: true,
        controlFlowFlatteningThreshold: 0.5,
        deadCodeInjection: true,
        deadCodeInjectionThreshold: 0.3,
        stringArray: true,
        stringArrayThreshold: 0.75,
        stringArrayRotate: true,
        stringArrayShuffle: true,
        stringArrayIndexShift: true,
        stringArrayCallsTransform: true,
        stringArrayCallsTransformThreshold: 0.5,
        stringArrayWrappersCount: 1,
        stringArrayWrappersChainedCalls: true,
        identifierNamesGenerator: 'hexadecimal',
        disableConsoleOutput: true,
        renameGlobals: false,
        selfDefending: false,
        debugProtection: false,
        splitStrings: false,
        numbersToExpressions: false,
        ignoreImports: true,
        unicodeEscapeSequence: false,
        target: 'browser',
      },
    }),
  ],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
