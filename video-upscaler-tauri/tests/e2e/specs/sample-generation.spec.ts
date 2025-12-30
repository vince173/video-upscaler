/**
 * Sample Generation Flow Tests
 *
 * Tests the 10-second sample generation feature
 */

import { TestHelpers } from '../helpers/test-helpers';

describe('Sample Generation Flow', () => {
  let helpers: TestHelpers;

  beforeEach(async () => {
    helpers = new TestHelpers(browser);
  });

  it('should generate 10-second sample from test video', async () => {
    // Step 1: Select a test video
    await helpers.selectVideoForTest('sample-10s.mp4');

    // Verify file is displayed
    const fileName = await browser.$('#selected-file').getText();
    expect(fileName).toContain('sample-10s.mp4');

    // Verify video player is shown
    const playerSection = browser.$('#player-section');
    await expect(playerSection).toBeDisplayed();

    // Verify controls are shown
    const controlsSection = browser.$('#controls-section');
    await expect(controlsSection).toBeDisplayed();

    // Step 2: Click generate sample button
    await helpers.clickAndWait('generate-sample-btn');

    // Step 3: Verify progress section appears
    await helpers.waitForProgressSection();

    const statusText = await browser.$('#status-text').getText();
    expect(statusText).toMatch(/generating|sample/i);

    // Step 4: Wait for completion
    await helpers.waitForProcessingComplete();

    // Step 5: Verify enhanced video tab appears
    const enhancedTabVisible = await helpers.isEnhancedTabVisible();
    expect(enhancedTabVisible).toBe(true);

    // Step 6: Verify buttons are re-enabled
    const generateEnabled = await helpers.isButtonEnabled('generate-sample-btn');
    const processEnabled = await helpers.isButtonEnabled('process-full-btn');
    expect(generateEnabled).toBe(true);
    expect(processEnabled).toBe(true);
  });

  it('should show progress updates during generation', async () => {
    await helpers.selectVideoForTest('sample-10s.mp4');
    await browser.$('#generate-sample-btn').click();

    // Check progress updates
    await helpers.waitForProgressSection();

    let lastProgress = 0;
    const maxChecks = 5;

    for (let i = 0; i < maxChecks; i++) {
      await browser.pause(3000);
      const current = await helpers.getProgressPercentage();

      // Progress should increase or stay the same (never decrease)
      expect(current).toBeGreaterThanOrEqual(lastProgress);
      lastProgress = current;

      // If complete, stop checking
      if (current >= 100) {
        break;
      }
    }

    await helpers.waitForProcessingComplete();

    // Final progress should be 100%
    const finalProgress = await helpers.getProgressPercentage();
    expect(finalProgress).toBe(100);
  });

  it('should disable buttons during processing', async () => {
    await helpers.selectVideoForTest('sample-10s.mp4');
    await browser.$('#generate-sample-btn').click();

    // Verify buttons are disabled while processing
    const generateEnabled = await helpers.isButtonEnabled('generate-sample-btn');
    const processEnabled = await helpers.isButtonEnabled('process-full-btn');
    const selectEnabled = await helpers.isButtonEnabled('select-video-btn');

    expect(generateEnabled).toBe(false);
    expect(processEnabled).toBe(false);
    expect(selectEnabled).toBe(false);

    // Wait for completion
    await helpers.waitForProcessingComplete();

    // Verify buttons are re-enabled after completion
    const genEnabled = await helpers.isButtonEnabled('generate-sample-btn');
    const procEnabled = await helpers.isButtonEnabled('process-full-btn');
    const selEnabled = await helpers.isButtonEnabled('select-video-btn');

    expect(genEnabled).toBe(true);
    expect(procEnabled).toBe(true);
    expect(selEnabled).toBe(true);
  });

  it('should allow switching to enhanced video after generation', async () => {
    await helpers.selectVideoForTest('sample-10s.mp4');
    await browser.$('#generate-sample-btn').click();
    await helpers.waitForProcessingComplete();

    // Switch to enhanced tab
    await helpers.switchToEnhancedTab();

    // Verify enhanced video is shown
    const enhancedVideo = browser.$('#enhanced-video');
    await expect(enhancedVideo).toBeDisplayed();

    // Verify original video is hidden
    const originalVideo = browser.$('#original-video');
    const isVisible = await originalVideo.isDisplayed();
    expect(isVisible).toBe(false);

    // Switch back to original
    await helpers.switchToOriginalTab();

    // Verify original video is shown again
    await expect(originalVideo).toBeDisplayed();
  });
});
