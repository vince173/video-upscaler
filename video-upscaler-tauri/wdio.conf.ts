/**
 * WebdriverIO Configuration for Tauri E2E Testing
 *
 * Based on @crabnebula/tauri-driver documentation:
 * https://www.npmjs.com/package/@crabnebula/tauri-driver
 */

import { spawn, spawnSync } from 'child_process';
import path from 'path';
import { waitTauriDriverReady } from '@crabnebula/tauri-driver';

// Keep track of the tauri-driver child process
let tauriDriver: ReturnType<typeof spawn>;
let killedTauriDriver = false;

// Set your application path
// Note: npm run tauri build --debug builds in release folder, not debug
const applicationPath = path.join(
  process.cwd(),
  'src-tauri',
  'target',
  'release',
  process.platform === 'win32' ? 'video-upscaler-tauri.exe' : 'video-upscaler-tauri'
);

export const config: WebdriverIO.Config = {
  // ====================
  // WebDriver Connection
  // ====================
  hostname: '127.0.0.1',
  port: 4444,

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
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        application: applicationPath,
      },
    },
  ],

  // ====================
  // Test Timeout
  // ====================
  waitforTimeout: 120000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 0,
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
    [
      'junit',
      {
        outputDir: './test-results/junit-results',
        outputFileFormat: (options: { cid: string; capabilities: any }) => {
          return `results-${options.cid}.xml`;
        },
      },
    ],
  ],

  // ====================
  // Automation Hooks
  // ====================

  // Build the Tauri app (frontend + backend) for testing
  // Using npm run tauri build ensures beforeBuildCommand runs
  onPrepare: async () => {
    // Build the Tauri app
    spawnSync('npm', ['run', 'tauri', 'build', '--debug'], {
      stdio: 'inherit',
      shell: true,
    });

    // Start tauri-driver once for all tests
    tauriDriver = spawn('tauri-driver', [], {
      stdio: [null, process.stdout, process.stderr],
      shell: true,
    });

    tauriDriver.on('error', (error) => {
      console.error('tauri-driver error:', error);
      process.exit(1);
    });

    tauriDriver.on('exit', (code) => {
      if (!killedTauriDriver) {
        console.error('tauri-driver exited with code:', code);
        process.exit(1);
      }
    });

    // Wait for tauri-driver to initialize its proxy server
    await waitTauriDriverReady();
  },

  // Clean up the `tauri-driver` process after all tests complete
  onComplete: () => {
    closeTauriDriver();
  },

  // ====================
  // Logging
  // ====================
  logLevel: 'info',
  stderr: true,

  // ====================
  // Additional Options
  // ====================
  bail: 0,
  screenshotPath: './test-results/screenshots',
};

function closeTauriDriver() {
  killedTauriDriver = true;
  tauriDriver?.kill();
}

function onShutdown(fn: () => void) {
  const cleanup = () => {
    try {
      fn();
    } finally {
      process.exit(0);
    }
  };

  process.on('exit', cleanup);
  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('SIGHUP', cleanup);
  process.on('SIGBREAK', cleanup);
}

onShutdown(() => {
  closeTauriDriver();
});
