import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: '{{test_dir}}',
  use: {
    baseURL: '{{url}}',
  },
});
