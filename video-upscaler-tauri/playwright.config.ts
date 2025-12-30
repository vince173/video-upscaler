import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for E2E testing
 *
 * Key configurations:
 * - 120s timeout per test (video processing takes time)
 * - Single worker (FFmpeg resource intensive)
 * - Screenshots/video on failure for debugging
 * - HTML and JSON reporters for CI/CD
 *
 * Prerequisites:
 * - Vite dev server must be running on http://localhost:1420
 *   Run: npm run dev
 */
export default defineConfig({
  testDir: './tests/e2e/specs',
  timeout: 120000, // 2 minutes per test (video processing takes time)
  expect: {
    timeout: 30000,
  },
  fullyParallel: false, // Run tests sequentially (FFmpeg resource intensive)
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Single worker to avoid FFmpeg conflicts
  outputDir: 'test-results/artifacts', // Separate artifacts from reports
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['list'],
  ],
  use: {
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    actionTimeout: 30000,
    // Base URL for Vite dev server
    baseURL: 'http://localhost:1420',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
