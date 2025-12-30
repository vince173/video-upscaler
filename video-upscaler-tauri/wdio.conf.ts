/**
 * WebdriverIO Configuration for Tauri E2E Testing
 *
 * This configuration automatically:
 * 1. Builds the Tauri app
 * 2. Starts tauri-driver
 * 3. Runs tests against the Tauri window
 * 4. Cleans up tauri-driver process
 */

import { spawn, spawnSync } from 'child_process';
import path from 'path';
import os from 'os';

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
    maxInstances: 1,
    'tauri:options': {
      // Path to the built Tauri application binary
      // On Windows: src-tauri/target/release/video-upscaler-tauri.exe
      // On Linux/macOS: src-tauri/target/release/video-upscaler-tauri
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
  // 120 seconds per test (video processing takes time)
  waitforTimeout: 120000,
  connectionRetryTimeout: 120000,
  frameworkTimeout: 300000,

  // ====================
  // Framework
  // ====================
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 120000, // 2 minutes per test
    retries: process.env.CI ? 2 : 0,
  },

  // ====================
  // Reporters
  // ====================
  reporters: [
    'spec',
    ['junit', {
      outputDir: './test-results/junit-results',
      outputFileFormat: 'results-[hash].xml',
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

    // Find tauri-driver in PATH or cargo bin
    const tauriDriverPath = process.platform === 'win32'
      ? 'tauri-driver.exe'
      : 'tauri-driver';

    tauriDriver = spawn(tauriDriverPath, [], {
      stdio: [null, process.stdout, process.stderr],
      shell: true,
    });

    // Wait for tauri-driver to be ready
    return new Promise<void>((resolve) => {
      tauriDriver.stdout?.once('data', () => {
        console.log('✅ tauri-driver is ready');
        resolve();
      });

      // Timeout after 10 seconds
      setTimeout(() => {
        console.log('⏱️ tauri-driver timeout, assuming ready');
        resolve();
      }, 10000);
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
  maxInstances: 1, // Run tests sequentially (FFmpeg resource intensive)
  bail: 0, // Don't stop on failure
  screenshotPath: './test-results/screenshots',
};
