import { defineConfig, devices } from '@playwright/test';
import path from 'node:path';

const repoRoot = path.resolve(__dirname, '../../../..');
const RELEASE_SERVER_URL = 'http://127.0.0.1:4002';
const reuseExistingServer = process.env['PLAYWRIGHT_REUSE_SERVER'] === '1';

export default defineConfig({
  testDir: './e2e-release',
  timeout: 120_000,
  expect: { timeout: 15_000 },
  fullyParallel: false,
  workers: 1,
  forbidOnly: !!process.env['CI'],
  retries: process.env['CI'] ? 1 : 0,
  reporter: [
    ['list'],
    ['html', { outputFolder: 'playwright-report-release', open: 'never' }],
  ],
  outputDir: 'test-results-release',

  use: {
    ...devices['Desktop Chrome'],
    channel: 'chrome',
    headless: false,
    trace: 'on-first-retry',
    launchOptions: {
      args: [
        '--enable-unsafe-webgpu',
        '--use-angle=swiftshader',
        '--disable-gpu-sandbox',
        '--ignore-gpu-blocklist',
      ],
    },
  },

  webServer: {
    command: 'install-ctl prepare spec-viewer && install-ctl start spec-viewer --foreground',
    url: RELEASE_SERVER_URL,
    cwd: repoRoot,
    reuseExistingServer,
    timeout: 300_000,
  },
});