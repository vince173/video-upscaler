/**
 * Test Fixture Generator
 *
 * Generates reproducible test videos using FFmpeg's testsrc filter.
 * These videos are used for E2E testing of the video upscaler application.
 */

import { execaCommand } from 'execa';
import fs from 'fs-extra';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const FIXTURES_DIR = path.join(__dirname, 'videos');

/**
 * Check if FFmpeg is available on the system
 */
async function checkFFmpegAvailable(): Promise<boolean> {
  try {
    await execaCommand('ffmpeg -version', { stdio: 'pipe' });
    return true;
  } catch {
    return false;
  }
}

/**
 * Download a sample video from a reliable source (Big Buck Bunny clip)
 * This is used as a fallback when FFmpeg is not available
 */
async function downloadSampleVideo(filename: string, url: string): Promise<void> {
  const outputPath = path.join(FIXTURES_DIR, filename);

  console.log(`Downloading ${filename} from ${url}...`);

  try {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const arrayBuffer = await response.arrayBuffer();
    const buffer = Buffer.from(arrayBuffer);
    await fs.writeFile(outputPath, buffer);

    const stats = await fs.stat(outputPath);
    console.log(`✓ Downloaded ${filename} (${(stats.size / 1024 / 1024).toFixed(2)} MB)`);
  } catch (error) {
    console.error(`✗ Failed to download ${filename}:`, error);
    throw error;
  }
}

/**
 * Generate a test video using FFmpeg testsrc
 */
async function generateTestVideo(
  filename: string,
  duration: number,
  size: string,
  fps: number
): Promise<void> {
  const outputPath = path.join(FIXTURES_DIR, filename);

  console.log(`Generating ${filename} (${duration}s at ${size}, ${fps}fps)...`);

  try {
    // FFmpeg testsrc generates test patterns
    await execaCommand(
      `ffmpeg -f lavfi -i testsrc=duration=${duration}:size=${size}:rate=${fps} ` +
      `-c:v libx264 -preset ultrafast -crf 23 -pix_fmt yuv420p ` +
      `-an "${outputPath}"`,
      { stdio: 'pipe' }
    );

    console.log(`✓ Generated ${filename}`);

    // Verify file was created
    const stats = await fs.stat(outputPath);
    console.log(`  Size: ${(stats.size / 1024 / 1024).toFixed(2)} MB`);
  } catch (error) {
    console.error(`✗ Failed to generate ${filename}:`, error);
    throw error;
  }
}

/**
 * Generate a corrupted/invalid video file for error testing
 */
async function generateInvalidVideo(): Promise<void> {
  const outputPath = path.join(FIXTURES_DIR, 'invalid.mp4');

  console.log('Generating invalid.mp4 (corrupted video for error testing)...');

  // Write random bytes to simulate a corrupted file
  const buffer = Buffer.from('INVALID_VIDEO_DATA_CORRUPTED_FILE_HEADER');
  await fs.writeFile(outputPath, buffer);

  console.log('✓ Generated invalid.mp4');
}

/**
 * Generate all test fixtures
 */
async function generateAllFixtures(): Promise<void> {
  console.log('Generating test video fixtures...\n');

  // Ensure fixtures directory exists
  await fs.ensureDir(FIXTURES_DIR);

  const hasFFmpeg = await checkFFmpegAvailable();

  if (!hasFFmpeg) {
    console.warn('\n⚠️  FFmpeg not found on system!');
    console.warn('Attempting to download sample videos from online sources...\n');
    console.warn('To generate custom test videos, install FFmpeg:');
    console.warn('  Windows: choco install ffmpeg');
    console.warn('  macOS:   brew install ffmpeg');
    console.warn('  Linux:   sudo apt install ffmpeg\n');
  }

  try {
    if (hasFFmpeg) {
      // Generate test videos using FFmpeg
      await Promise.all([
        // Small 10-second video for quick tests
        generateTestVideo('sample-10s.mp4', 10, '320x240', 30),

        // Medium 30-second video for testing progress and cancellation
        generateTestVideo('sample-30s.mp4', 30, '640x360', 30),
      ]);
    } else {
      // Download sample videos as fallback
      console.log('Using fallback: downloading sample videos...\n');

      // Download small test video (Big Buck Bunny short clip)
      await downloadSampleVideo(
        'sample-10s.mp4',
        'https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/360/Big_Buck_Bunny_360_10s_1MB.mp4'
      );

      // Download medium test video
      await downloadSampleVideo(
        'sample-30s.mp4',
        'https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/720/Big_Buck_Bunny_720_10s_5MB.mp4'
      );
    }

    // Generate invalid video (doesn't require FFmpeg)
    await generateInvalidVideo();

    console.log('\n✓ All fixtures generated successfully!');
    console.log(`Fixture location: ${FIXTURES_DIR}`);
  } catch (error) {
    console.error('\n❌ Error generating fixtures:', error);
    console.error('\nPlease manually download sample videos to:');
    console.error(`  ${FIXTURES_DIR}`);
    console.error('\nRequired files:');
    console.error('  - sample-10s.mp4 (short video for quick tests)');
    console.error('  - sample-30s.mp4 (longer video for progress/cancellation tests)');
    console.error('  - invalid.mp4 (corrupted file for error testing)');
    throw error;
  }
}

// Run if executed directly
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  generateAllFixtures()
    .then(() => {
      process.exit(0);
    })
    .catch((error) => {
      process.exit(1);
    });
}

export { generateAllFixtures };
