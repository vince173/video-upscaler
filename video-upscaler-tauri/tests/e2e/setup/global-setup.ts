/**
 * Global test setup for Tauri E2E tests
 *
 * This file handles:
 * - Starting the Tauri dev server before tests
 * - Cleaning up (stopping the server) after tests
 */

import { FullConfig } from '@playwright/test';
import { spawn, ChildProcess } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

let tauriProcess: ChildProcess | null = null;

async function globalSetup(config: FullConfig) {
  console.log('🚀 Starting Tauri dev server for E2E tests...');

  // Check if dev server is already running
  try {
    const response = await fetch('http://localhost:1420', { method: 'HEAD', signal: AbortSignal.timeout(2000) });
    if (response.ok) {
      console.log('✅ Dev server already running at http://localhost:1420');
      return;
    }
  } catch {
    // Server not running, start it
  }

  // Start Tauri dev server
  const tauriDev = spawn('npm', ['run', 'tauri', 'dev'], {
    stdio: 'pipe',
    shell: true,
    cwd: path.resolve(__dirname, '../../..'),
    env: {
      ...process.env,
      // Enable test mode in the app
      TAURI_ENV_MODE: 'test',
    },
  });

  tauriProcess = tauriDev;

  // Log output for debugging
  tauriDev.stdout?.on('data', (data) => {
    const output = data.toString();
    // Only log important messages
    if (output.includes('ready') || output.includes('listening') || output.includes('error')) {
      console.log(`[Tauri] ${output.trim()}`);
    }
  });

  tauriDev.stderr?.on('data', (data) => {
    console.error(`[Tauri Error] ${data.toString().trim()}`);
  });

  // Wait for server to be ready
  const maxWaitTime = 120000; // 2 minutes
  const startTime = Date.now();

  while (Date.now() - startTime < maxWaitTime) {
    try {
      const response = await fetch('http://localhost:1420', { method: 'HEAD', signal: AbortSignal.timeout(1000) });
      if (response.ok) {
        console.log('✅ Tauri dev server is ready at http://localhost:1420');
        return;
      }
    } catch {
      // Server not ready yet, wait and retry
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  }

  throw new Error('Tauri dev server failed to start within 2 minutes');
}

async function globalTeardown(config: FullConfig) {
  console.log('🛑 Stopping Tauri dev server...');

  if (tauriProcess) {
    // Kill the process tree on Windows
    if (process.platform === 'win32') {
      spawn('taskkill', ['/pid', String(tauriProcess.pid), '/T', '/F'], {
        stdio: 'ignore',
      });
    } else {
      tauriProcess.kill('SIGTERM');
    }

    // Wait for process to exit
    await new Promise((resolve) => setTimeout(resolve, 2000));
    console.log('✅ Tauri dev server stopped');
  }
}

export default globalSetup;
export { globalTeardown };
