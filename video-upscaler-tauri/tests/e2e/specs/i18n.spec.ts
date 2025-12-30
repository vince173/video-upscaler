/**
 * Internationalization (i18n) Tests
 *
 * Tests language switching functionality and translation correctness
 */

import { TestHelpers } from '../helpers/test-helpers';

describe('Language Switching', () => {
  let helpers: TestHelpers;

  beforeEach(async () => {
    helpers = new TestHelpers(browser);
  });

  it('should switch to English and verify all UI text', async () => {
    await helpers.selectLanguage('en');

    // Verify page title
    const title = await browser.getTitle();
    expect(title).toMatch(/Video Quality Enhancer/i);

    // Verify header
    const header = browser.$('[data-i18n="header.title"]');
    const headerText = await header.getText();
    expect(headerText).toMatch(/Video Quality Enhancer/i);

    // Verify select button
    const selectBtn = browser.$('[data-i18n="file_selection.select_button"]');
    const selectBtnText = await selectBtn.getText();
    expect(selectBtnText).toMatch(/Select Video File/i);

    // Verify tabs
    const originalTab = browser.$('[data-i18n="tabs.original"]');
    const originalTabText = await originalTab.getText();
    expect(originalTabText).toMatch(/Original Video/i);
  });

  it('should switch to Chinese and verify all UI text', async () => {
    await helpers.selectLanguage('zh');

    // Verify page title
    const title = await browser.getTitle();
    expect(title).toMatch(/视频画质增强工具/);

    // Verify header
    const header = browser.$('[data-i18n="header.title"]');
    const headerText = await header.getText();
    expect(headerText).toMatch(/视频画质增强工具/);

    // Verify select button
    const selectBtn = browser.$('[data-i18n="file_selection.select_button"]');
    const selectBtnText = await selectBtn.getText();
    expect(selectBtnText).toMatch(/选择视频文件/);

    // Verify tabs
    const originalTab = browser.$('[data-i18n="tabs.original"]');
    const originalTabText = await originalTab.getText();
    expect(originalTabText).toMatch(/原始视频/);
  });

  it('should persist language selection on page reload', async () => {
    // Switch to English
    await helpers.selectLanguage('en');

    // Reload page
    await browser.refresh();

    // Verify language persisted
    const langSelect = browser.$('#language-select');
    const value = await langSelect.getValue();
    expect(value).toBe('en');

    const title = await browser.getTitle();
    expect(title).toMatch(/Video Quality Enhancer/i);
  });

  it('should translate error messages correctly', async () => {
    // Switch to English
    await helpers.selectLanguage('en');

    // Try to process without selecting a file
    await browser.$('#process-full-btn').click();

    let errorText = await browser.$('#error-text').getText();
    expect(errorText).toMatch(/select.*video/i);

    // Clear error
    await helpers.clearError();

    // Switch to Chinese
    await helpers.selectLanguage('zh');

    // Try to process without selecting a file again
    await browser.$('#process-full-btn').click();

    errorText = await browser.$('#error-text').getText();
    expect(errorText).toMatch(/选择.*视频/i);
  });

  it('should translate progress status messages', async () => {
    await helpers.selectVideoForTest('sample-10s.mp4');

    // Test in English
    await helpers.selectLanguage('en');
    await browser.$('#generate-sample-btn').click();

    let statusText = await browser.$('#status-text').getText();
    expect(statusText).toMatch(/generating/i);

    // Cancel and try in Chinese
    await browser.$('#cancel-btn').click();
    await browser.pause(1000);

    await helpers.selectLanguage('zh');
    await browser.$('#generate-sample-btn').click();

    statusText = await browser.$('#status-text').getText();
    expect(statusText).toMatch(/生成/i);

    // Clean up
    await browser.$('#cancel-btn').click();
  });

  it('should translate button labels during processing', async () => {
    await helpers.selectVideoForTest('sample-10s.mp4');

    // Check cancel button in English
    await helpers.selectLanguage('en');
    await browser.$('#generate-sample-btn').click();

    let cancelBtn = browser.$('[data-i18n="progress.cancel_button"]');
    let cancelBtnText = await cancelBtn.getText();
    expect(cancelBtnText).toMatch(/Cancel/i);

    // Cancel and check in Chinese
    await browser.$('#cancel-btn').click();
    await browser.pause(1000);

    await helpers.selectLanguage('zh');
    await browser.$('#generate-sample-btn').click();

    cancelBtn = browser.$('[data-i18n="progress.cancel_button"]');
    cancelBtnText = await cancelBtn.getText();
    expect(cancelBtnText).toMatch(/取消/i);

    // Clean up
    await browser.$('#cancel-btn').click();
  });

  it('should switch language during processing', async () => {
    await helpers.selectVideoForTest('sample-30s.mp4');
    await browser.$('#process-full-btn').click();

    // Wait for progress to start
    await helpers.waitForProgressSection();

    // Switch language while processing
    await helpers.selectLanguage('zh');

    // Verify UI updated (check error or progress section)
    // Language should update even during processing
    const langSelect = browser.$('#language-select');
    const value = await langSelect.getValue();
    expect(value).toBe('zh');

    // Cancel to clean up
    await browser.$('#cancel-btn').click();
  });
});
