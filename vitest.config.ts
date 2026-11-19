import { configDefaults, defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["{apps,packages}/*/**/*.{test,spec}.?(c|m)[jt]s?(x)"],
    exclude: [...configDefaults.exclude, "**/dist/**", "**/e2e/**", "**/target/**"],
  },
});
