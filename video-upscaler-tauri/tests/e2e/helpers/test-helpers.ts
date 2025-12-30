/**
 * Test Helper Utilities
 *
 * Common helper functions for E2E tests to interact with the
 * video upscaler application UI using WebdriverIO with Tauri driver.
 */

import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * TestHelpers class provides utility methods for common E2E test operations
 * using WebdriverIO with Tauri driver support
 */
export class TestHelpers {
  constructor(private browser: WebdriverIO.Browser) {}

  /**
   * Select a video file for testing using direct UI interaction
   * This approach uses the test mode in main.ts
   *
   * @param filename - Name of the test video file in fixtures/videos
   */
  async selectVideoForTest(filename: string): Promise<void> {
    const fixturesDir = path.resolve(__dirname, '../fixtures/videos');
    const filePath = path.join(fixturesDir, filename);

    // Verify the test file exists
    if (!fs.existsSync(filePath)) {
      throw new Error(`Test fixture not found: ${filePath}`);
    }

    // Set up the test file path BEFORE clicking the button
    await this.browser.execute(
      (testPath: string) => {
        // Set the test file path that main.ts will check
        (window as any).__tauri_test_file_path = testPath;
        console.log('Test file path set:', testPath);
      },
      filePath
    );

    // Click the select video button - the test mode will intercept
    await this.browser.$('#select-video-btn').click();

    // Wait for the video to load in the player
    await this.browser.waitUntil(
      async () => {
        const src = await this.browser.$('#original-video').getAttribute('src');
        return src !== null && src !== '';
      },
      { timeout: 10000, timeoutMsg: 'Video did not load' }
    );

    // Verify the file name is displayed
    const fileName = await this.browser.$('#selected-file').getText();
    expect(fileName).toContain(filename);
  }

  /**
   * Wait for video processing to complete
   * Checks that progress section disappears
   *
   * @param timeout - Maximum time to wait in milliseconds
   */
  async waitForProcessingComplete(timeout = 120000): Promise<void> {
    await this.browser.waitUntil(
      async () => {
        const isHidden = await this.browser.execute(() => {
          const progressSection = document.querySelector<HTMLDivElement>('#progress-section');
          if (!progressSection) return true;
          const style = window.getComputedStyle(progressSection);
          return style.display === 'none' || style.display === '';
        });
        return isHidden === true;
      },
      { timeout, timeoutMsg: 'Processing did not complete' }
    );
  }

  /**
   * Get current progress percentage from UI
   *
   * @returns Progress percentage (0-100)
   */
  async getProgressPercentage(): Promise<number> {
    const text = await this.browser.$('#progress-text').getText();
    const match = text?.match(/(\d+(?:\.\d+)?)%/);
    return match ? parseFloat(match[1]) : 0;
  }

  /**
   * Check if error is displayed
   *
   * @returns True if error section is visible
   */
  async hasError(): Promise<boolean> {
    return await this.browser.$('#error-section').isDisplayed();
  }

  /**
   * Get error message text
   *
   * @returns Error message text
   */
  async getErrorMessage(): Promise<string> {
    return (await this.browser.$('#error-text').getText()) || '';
  }

  /**
   * Click a button and wait for UI state to update
   *
   * @param buttonId - ID of the button to click
   */
  async clickAndWait(buttonId: string): Promise<void> {
    await this.browser.$(`#${buttonId}`).click();
    await this.browser.pause(500); // Wait for UI to update
  }

  /**
   * Check if a button is enabled
   *
   * @param buttonId - ID of the button to check
   * @returns True if button is enabled
   */
  async isButtonEnabled(buttonId: string): Promise<boolean> {
    return await this.browser.$(`#${buttonId}`).isEnabled();
  }

  /**
   * Wait for progress section to appear
   */
  async waitForProgressSection(): Promise<void> {
    await this.browser.waitUntil(
      async () => {
        const isVisible = await this.browser.execute(() => {
          const progressSection = document.querySelector<HTMLDivElement>('#progress-section');
          if (!progressSection) return false;
          const style = window.getComputedStyle(progressSection);
          return style.display !== 'none' && style.display !== '';
        });
        return isVisible === true;
      },
      { timeout: 10000, timeoutMsg: 'Progress section did not appear' }
    );
  }

  /**
   * Get current status text
   *
   * @returns Status text content
   */
  async getStatusText(): Promise<string> {
    return (await this.browser.$('#status-text').getText()) || '';
  }

  /**
   * Select a language from the dropdown
   *
   * @param langCode - Language code ('en' or 'zh')
   */
  async selectLanguage(langCode: string): Promise<void> {
    await this.browser.$('#language-select').selectByAttribute('value', langCode);
    // Wait for translations to apply
    await this.browser.pause(500);
  }

  /**
   * Close error section by clicking the select button (clears error)
   */
  async clearError(): Promise<void> {
    await this.browser.$('#select-video-btn').click();
    await this.browser.pause(100);
  }

  /**
   * Check if enhanced video tab is visible
   *
   * @returns True if enhanced tab is visible
   */
  async isEnhancedTabVisible(): Promise<boolean> {
    return await this.browser.$('#enhanced-tab-btn').isDisplayed();
  }

  /**
   * Switch to the enhanced video tab
   */
  async switchToEnhancedTab(): Promise<void> {
    await this.browser.$('#enhanced-tab-btn').click();
    await this.browser.pause(500);
  }

  /**
   * Switch to the original video tab
   */
  async switchToOriginalTab(): Promise<void> {
    await this.browser.$('.tab-btn[data-tab="original"]').click();
    await this.browser.pause(500);
  }
}

/**
 * Sleep utility for waiting in tests
 *
 * @param ms - Milliseconds to sleep
 */
export const sleep = (ms: number): Promise<void> =>
  new Promise((resolve) => setTimeout(resolve, ms));
