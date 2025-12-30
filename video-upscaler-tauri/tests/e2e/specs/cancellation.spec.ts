/**
 * Cancellation Flow Tests
 *
 * Tests the cancellation feature and cleanup behavior
 */

import { TestHelpers } from '../helpers/test-helpers';

describe('Cancellation Flow', () => {
  let helpers: TestHelpers;

  beforeEach(async () => {
    helpers = new TestHelpers(browser);
  });

  it('should cancel processing and return to ready state', async () => {
    // Use longer video to have time to cancel
    await helpers.selectVideoForTest('sample-30s.mp4');

    // Start processing
    await browser.$('#process-full-btn').click();

    // Wait for progress to start
    await helpers.waitForProgressSection();

    // Wait for some progress (at least 5%)
    await browser.waitUntil(
      async () => {
        const text = await browser.execute(() => {
          return document.querySelector('#progress-text')?.textContent;
        });
        const match = text?.match(/(\d+)%/);
        return match && parseInt(match[1]) > 5;
      },
      { timeout: 30000, timeoutMsg: 'Progress did not reach 5%' }
    );

    // Click cancel
    await browser.$('#cancel-btn').click();

    // Verify progress section disappears
    const progressSection = browser.$('#progress-section');
    await expect(progressSection).not.toBeDisplayed();

    // Verify buttons are re-enabled
    const processEnabled = await helpers.isButtonEnabled('process-full-btn');
    const generateEnabled = await helpers.isButtonEnabled('generate-sample-btn');
    expect(processEnabled).toBe(true);
    expect(generateEnabled).toBe(true);
  });

  it('should cancel during sample generation', async () => {
    await helpers.selectVideoForTest('sample-30s.mp4');
    await browser.$('#generate-sample-btn').click();

    // Wait for progress
    await helpers.waitForProgressSection();

    // Cancel quickly
    await browser.$('#cancel-btn').click();

    // Verify cleanup
    await expect(browser.$('#progress-section')).not.toBeDisplayed();
    await expect(browser.$('#error-section')).not.toBeDisplayed();
  });

  it('should allow re-processing after cancellation', async () => {
    await helpers.selectVideoForTest('sample-30s.mp4');

    // First attempt - cancel it
    await browser.$('#process-full-btn').click();
    await helpers.waitForProgressSection();
    await browser.pause(3000);
    await browser.$('#cancel-btn').click();

    // Wait for cleanup
    await browser.pause(1000);

    // Second attempt should work
    await browser.$('#generate-sample-btn').click();
    await helpers.waitForProgressSection();

    const progressVisible = await browser.$('#progress-section').isDisplayed();
    expect(progressVisible).toBe(true);

    // Clean up - cancel this one too
    await browser.$('#cancel-btn').click();
  });

  it('should handle multiple rapid cancel clicks', async () => {
    await helpers.selectVideoForTest('sample-30s.mp4');
    await browser.$('#process-full-btn').click();
    await helpers.waitForProgressSection();

    // Click cancel multiple times rapidly
    await browser.$('#cancel-btn').click();
    await browser.$('#cancel-btn').click();
    await browser.$('#cancel-btn').click();

    // Should still return to ready state
    await expect(browser.$('#progress-section')).not.toBeDisplayed();
    await expect(browser.$('#error-section')).not.toBeDisplayed();

    const processEnabled = await helpers.isButtonEnabled('process-full-btn');
    expect(processEnabled).toBe(true);
  });

  it('should cancel when no file selected (graceful handling)', async () => {
    // Try to cancel when nothing is processing
    await browser.$('#cancel-btn').click();

    // Should not cause any errors
    // No progress section should appear
    await expect(browser.$('#progress-section')).not.toBeDisplayed();
  });
});
