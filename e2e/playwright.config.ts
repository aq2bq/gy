import { defineConfig, devices } from '@playwright/test';

/* One browser, one worker, no retries: a red scene is a real failure (d-244b).
   Traces and screenshots stay beside the failure; nothing is compared by
   pixels (the looks are for the master's eye). */
export default defineConfig({
  testDir: './tests',
  timeout: 30000,
  expect: { timeout: 5000 },
  workers: 1,
  retries: 0,
  reporter: [['list']],
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        trace: 'retain-on-failure',
        screenshot: 'only-on-failure',
      },
    },
  ],
});
