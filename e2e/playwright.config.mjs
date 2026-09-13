import {defineConfig} from '@playwright/test';
export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  workers: 1,
  retries: 0,
  forbidOnly: !!process.env.CI,
  timeout: 30000,
  expect: {timeout: 3000},
  reporter: [['list'], ['json', {outputFile: 'test-results/results.json'}]],
  use: {viewport: {width: 1400, height: 1000}, screenshot: 'only-on-failure', trace: 'retain-on-failure'},
  projects: [{name: 'chromium', use: {browserName: 'chromium'}}]
});
