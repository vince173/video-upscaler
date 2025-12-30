/**
 * WebdriverIO Configuration for Tauri E2E Testing
 *
 * This configuration:
 * 1. Builds the Tauri app
 * 2. Starts tauri-driver as a WebDriver server
 * 3. Runs tests against the Tauri window
 */

import { spawn, spawnSync } from 'child_process';
import path from 'path';

// Keep track of the tauri-driver child process
let tauriDriver: ReturnType<typeof spawn>;

export const config: WebdriverIO.Config = {
  // ====================
  // Runner Configuration
  // ====================
  runner: 'local',
  autoCompileOpts: {
    autoCompile: true,
    tsNodeOpts: {
      project: '.',
      transpileOnly: true,
    },
  },

  // ====================
  // Test Files
  // ====================
  specs: [
    './tests/e2e/specs/**/*.ts',
  ],
  exclude: [],

  // ====================
  // Capabilities
  // ====================
  capabilities: [{
    browserName: 'tauri',
    'tauri:options': {
      application: path.join(
        process.cwd(),
        'src-tauri',
        'target',
        'release',
        process.platform === 'win32' ? 'video-upscaler-tauri.exe' : 'video-upscaler-tauri'
      ),
    },
  }],

  // ====================
  // Test Timeout
  // ====================
  waitforTimeout: 120000,
  connectionRetryTimeout: 120000,
  frameworkTimeout: 300000,

  // ====================
  // Framework
  // ====================
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 120000,
    retries: process.env.CI ? 2 : 0,
  },

  // ====================
  // Reporters
  // ====================
  reporters: [
    'spec',
    ['junit', {
      outputDir: './test-results/junit-results',
      outputFileFormat: (options: { cid: string; capabilities: any }) => {
        return `results-${options.cid}.xml`;
      },
    }],
  ],

  // ====================
  // Automation Hooks
  // ====================

  // Build the Tauri app before tests start
  onPrepare: () => {
    console.log('🔨 Building Tauri app in release mode...');
    const result = spawnSync('cargo', ['build', '--release', '--manifest-path', 'src-tauri/Cargo.toml'], {
      stdio: 'inherit',
      shell: true,
    });

    if (result.status !== 0) {
      throw new Error('Failed to build Tauri app');
    }
    console.log('✅ Build complete');
  },

  // Start tauri-driver before the session
  beforeSession: () => {
    console.log('🚀 Starting tauri-driver...');

    const tauriDriverPath = process.platform === 'win32'
      ? 'tauri-driver.exe'
      : 'tauri-driver';

    tauriDriver = spawn(tauriDriverPath, [], {
      stdio: ['ignore', 'pipe', 'pipe'],
      shell: true,
    });

    // Wait for tauri-driver to be ready
    return new Promise<void>((resolve) => {
      let ready = false;
      const timeout = setTimeout(() => {
        if (!ready) {
          console.log('⏱️ tauri-driver timeout, assuming ready');
          ready = true;
          resolve();
        }
      }, 10000);

      tauriDriver.stdout?.on('data', (data) => {
        if (!ready && data.toString().includes('listening on')) {
          console.log('✅ tauri-driver is ready');
          ready = true;
          clearTimeout(timeout);
          resolve();
        }
      });

      tauriDriver.stderr?.on('data', (data) => {
        console.log('[tauri-driver stderr]', data.toString());
      });
    });
  },

  // Clean up tauri-driver after tests
  afterSession: () => {
    console.log('🛑 Stopping tauri-driver...');
    if (tauriDriver) {
      tauriDriver.kill();
      console.log('✅ tauri-driver stopped');
    }
  },

  // ====================
  // Logging
  // ====================
  logLevel: 'info',
  stderr: true,

  // ====================
  // Additional Options
  // ====================
  maxInstances: 1,
  bail: 0,
  screenshotPath: './test-results/screenshots',
};
