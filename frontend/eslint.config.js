import js from "@eslint/js"
import {defineConfig, globalIgnores} from "eslint/config"
import pluginVue from "eslint-plugin-vue"
import globals from "globals"
import tseslint from "typescript-eslint"

// 只启用正确性规则，不引入格式规则：现有代码未使用分号，格式统一留给后续独立任务
export default defineConfig(
  globalIgnores(["dist/**", "dist-ssr/**", "coverage/**"]),
  js.configs.recommended,
  tseslint.configs.recommended,
  pluginVue.configs["flat/essential"],
  {
    files: ["**/*.{ts,vue}"],
    languageOptions: {
      globals: globals.browser,
    },
  },
  {
    files: ["**/*.vue"],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
      },
    },
  },
  {
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        {argsIgnorePattern: "^_", caughtErrorsIgnorePattern: "^_"},
      ],
    },
  },
)
