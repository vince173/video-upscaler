import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

// State
let selectedFilePath: string | null = null;
let enhancedVideoPath: string | null = null;
let originalVideoObjectUrl: string | null = null;
let enhancedVideoObjectUrl: string | null = null;
let currentProcessingPath: string | null = null;
let isProcessing = false;
// Track sample files for cleanup
let sampleFiles: string[] = [];

// i18n State
let currentLanguage: string = "en";
let translations: Record<string, any> = {};

// DOM Elements - will be initialized after DOM is ready
let videoInput: HTMLInputElement;
let selectVideoBtn: HTMLButtonElement;
let selectedFileDiv: HTMLDivElement;
let playerSection: HTMLDivElement;
let controlsSection: HTMLDivElement;
let progressSection: HTMLDivElement;
let errorSection: HTMLDivElement;
let successSection: HTMLDivElement;
let originalVideo: HTMLVideoElement;
let enhancedVideo: HTMLVideoElement;
let generateSampleBtn: HTMLButtonElement;
let processFullBtn: HTMLButtonElement;
let cancelBtn: HTMLButtonElement;
let progressFill: HTMLDivElement;
let progressText: HTMLDivElement;
let statusText: HTMLDivElement;
let errorText: HTMLParagraphElement;
let successMessage: HTMLParagraphElement;
let successFilePath: HTMLParagraphElement;
let openFolderBtn: HTMLButtonElement;
let closeSuccessBtn: HTMLButtonElement;
let enhancedTabBtn: HTMLButtonElement;
let tabButtons: NodeListOf<Element>;
let languageSelect: HTMLSelectElement;

// Initialize
window.addEventListener("DOMContentLoaded", async () => {
  // Initialize DOM elements
  videoInput = document.getElementById("video-input") as HTMLInputElement;
  selectVideoBtn = document.getElementById("select-video-btn") as HTMLButtonElement;
  selectedFileDiv = document.getElementById("selected-file") as HTMLDivElement;
  playerSection = document.getElementById("player-section") as HTMLDivElement;
  controlsSection = document.getElementById("controls-section") as HTMLDivElement;
  progressSection = document.getElementById("progress-section") as HTMLDivElement;
  errorSection = document.getElementById("error-section") as HTMLDivElement;
  successSection = document.getElementById("success-section") as HTMLDivElement;
  originalVideo = document.getElementById("original-video") as HTMLVideoElement;
  enhancedVideo = document.getElementById("enhanced-video") as HTMLVideoElement;
  generateSampleBtn = document.getElementById("generate-sample-btn") as HTMLButtonElement;
  processFullBtn = document.getElementById("process-full-btn") as HTMLButtonElement;
  cancelBtn = document.getElementById("cancel-btn") as HTMLButtonElement;
  progressFill = document.getElementById("progress-fill") as HTMLDivElement;
  progressText = document.getElementById("progress-text") as HTMLDivElement;
  statusText = document.getElementById("status-text") as HTMLDivElement;
  errorText = document.getElementById("error-text") as HTMLParagraphElement;
  successMessage = document.getElementById("success-message") as HTMLParagraphElement;
  successFilePath = document.getElementById("success-file-path") as HTMLParagraphElement;
  openFolderBtn = document.getElementById("open-folder-btn") as HTMLButtonElement;
  closeSuccessBtn = document.getElementById("close-success-btn") as HTMLButtonElement;
  enhancedTabBtn = document.getElementById("enhanced-tab-btn") as HTMLButtonElement;
  tabButtons = document.querySelectorAll(".tab-btn");
  languageSelect = document.getElementById("language-select") as HTMLSelectElement;

  console.log("DOM Elements initialized");

  setupEventListeners();
  setupProgressListener();

  // Initialize i18n
  await initializeI18n();

  // Set up cleanup handler for window close
  window.addEventListener("beforeunload", async () => {
    if (sampleFiles.length > 0) {
      console.log("Cleaning up sample files:", sampleFiles);
      try {
        const deleted = await invoke<number>("cleanup_temp_files", { files: sampleFiles });
        console.log(`Cleaned up ${deleted} sample files`);
      } catch (error) {
        console.error("Failed to cleanup sample files:", error);
      }
    }
  });
});

// i18n Functions

async function initializeI18n() {
  try {
    // Get current language from backend
    currentLanguage = await invoke<string>("get_current_language");
    console.log("Current language:", currentLanguage);

    // Load translations
    await loadTranslations();

    // Set language selector value
    languageSelect.value = currentLanguage;

    // Apply translations to all elements
    applyTranslations();
  } catch (error) {
    console.error("Failed to initialize i18n:", error);
  }
}

async function loadTranslations() {
  try {
    translations = await invoke<Record<string, any>>("get_translations");
    console.log("Translations loaded:", translations);
  } catch (error) {
    console.error("Failed to load translations:", error);
  }
}

function t(key: string, replacements?: Record<string, string>): string {
  const keys = key.split(".");
  let value: any = translations;

  for (const k of keys) {
    if (value && typeof value === "object" && k in value) {
      value = value[k];
    } else {
      console.warn(`Translation key not found: ${key}`);
      return key;
    }
  }

  if (typeof value !== "string") {
    console.warn(`Translation value is not a string: ${key}`);
    return key;
  }

  // Apply replacements
  if (replacements) {
    for (const [placeholder, replacement] of Object.entries(replacements)) {
      value = value.replace(`{${placeholder}}`, replacement);
    }
  }

  return value;
}

function applyTranslations() {
  // Update all elements with data-i18n attribute
  const elements = document.querySelectorAll("[data-i18n]");
  elements.forEach((element) => {
    const key = element.getAttribute("data-i18n");
    if (key) {
      const translatedText = t(key);
      // Preserve emoji icons if present
      const emojiMatch = element.textContent?.match(/^[\p{Emoji}\s]+/u);
      if (emojiMatch) {
        element.textContent = emojiMatch[0] + " " + translatedText;
      } else {
        element.textContent = translatedText;
      }
    }
  });

  // Update page title
  document.title = t("app.title");

  // Update HTML lang attribute
  document.documentElement.lang = currentLanguage;
}

async function changeLanguage(langCode: string) {
  if (langCode === currentLanguage) return;

  try {
    await invoke("set_language_command", { langCode: langCode });
    currentLanguage = langCode;
    await loadTranslations();
    applyTranslations();
    console.log("Language changed to:", langCode);
  } catch (error) {
    console.error("Failed to change language:", error);
    showError(t("errors.select_failed", { error: String(error) }));
  }
}

// Set up real progress event listener from Rust backend
async function setupProgressListener() {
  await listen("video-progress", (event) => {
    const data = event.payload as { current: number; total: number; percentage: number };
    updateProgress(data.current, data.total, data.percentage);
  });
  console.log("Progress listener registered");
}

function setupEventListeners() {
  console.log("Setting up event listeners...");

  // Language selector
  languageSelect.addEventListener("change", (e) => {
    const target = e.target as HTMLSelectElement;
    changeLanguage(target.value);
  });

  // File selection
  selectVideoBtn.addEventListener("click", async (e) => {
    console.log("Button clicked!", e);
    try {
      // Test mode: check if a test file path was provided
      if ((window as any).__tauri_test_file_path) {
        const testPath = (window as any).__tauri_test_file_path;
        console.log("Test mode: using file path:", testPath);
        (window as any).__tauri_test_file_path = null; // Clear after use
        selectedFilePath = testPath;
        onFileSelected(testPath);
        return;
      }

      console.log("Calling open()...");
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Video",
            extensions: ["mp4", "avi", "mov", "mkv", "webm"]
          }
        ]
      });
      console.log("Selected file:", selected);

      if (selected && typeof selected === "string") {
        selectedFilePath = selected;
        onFileSelected(selected);
      }
    } catch (error) {
      console.error("Error in file selection:", error);
      showError(t("errors.select_failed", { error: String(error) }));
    }
  });

  // Tab switching
  tabButtons.forEach(btn => {
    btn.addEventListener("click", () => {
      const tab = (btn as HTMLButtonElement).getAttribute("data-tab");
      switchTab(tab || "original");
    });
  });

  // Process buttons
  generateSampleBtn.addEventListener("click", generateSample);
  processFullBtn.addEventListener("click", processFullVideo);

  // Cancel button
  cancelBtn.addEventListener("click", cancelProcessing);

  // Success section buttons
  openFolderBtn.addEventListener("click", async () => {
    if (enhancedVideoPath) {
      try {
        // Open the folder containing the enhanced video
        const path = enhancedVideoPath;
        // For Windows, use "explorer /select,<path>"
        // For macOS, use "open -R <path>"
        // For Linux, use "xdg-open <path>"
        await invoke("open_in_folder", { path: path });
      } catch (error) {
        console.error("Failed to open folder:", error);
      }
    }
  });

  closeSuccessBtn.addEventListener("click", () => {
    successSection.style.display = "none";
  });
}

async function onFileSelected(filePath: string) {
  // Display selected file
  const fileName = filePath.split(/[/\\]/).pop() || filePath;
  selectedFileDiv.textContent = `📹 ${fileName}`;

  // Show player and controls
  playerSection.style.display = "block";
  controlsSection.style.display = "block";

  // Load video using convertFileSrc
  console.log("Loading video:", filePath);

  try {
    // Convert file path to asset URL
    const assetUrl = await convertFileSrc(filePath);
    originalVideoObjectUrl = assetUrl;

    console.log("Asset URL:", assetUrl);

    originalVideo.src = assetUrl;
    originalVideo.load();

    // Add error handling
    originalVideo.onerror = (e) => {
      console.error("Video load error:", e);
      console.error("Video error code:", originalVideo.error?.code);
      console.error("Video error message:", originalVideo.error?.message);
    };

    originalVideo.onloadeddata = () => {
      console.log("Video loaded successfully!");
    };
  } catch (error) {
    console.error("Failed to load video:", error);
    showError(t("errors.load_failed", { error: String(error) }));
  }

  // Reset enhanced video
  enhancedVideo.style.display = "none";
  originalVideo.style.display = "block";
  enhancedVideoPath = null;

  // Hide enhanced tab until video is created
  enhancedTabBtn.style.display = "none";

  // Switch to original tab
  switchTab("original");
}

// Helper function to get video MIME type
function getVideoMimeType(filePath: string): string {
  const ext = filePath.split('.').pop()?.toLowerCase();
  const mimeTypes: { [key: string]: string } = {
    'mp4': 'video/mp4',
    'webm': 'video/webm',
    'ogg': 'video/ogg',
    'avi': 'video/x-msvideo',
    'mov': 'video/quicktime',
    'mkv': 'video/x-matroska'
  };
  return mimeTypes[ext || ''] || 'video/mp4';
}

function switchTab(tab: string) {
  tabButtons.forEach(btn => {
    if (btn.getAttribute("data-tab") === tab) {
      btn.classList.add("active");
    } else {
      btn.classList.remove("active");
    }
  });

  if (tab === "original") {
    originalVideo.style.display = "block";
    enhancedVideo.style.display = "none";
  } else if (tab === "enhanced") {
    if (enhancedVideoPath) {
      originalVideo.style.display = "none";
      enhancedVideo.style.display = "block";
    } else {
      // Show message or stay on original
      alert(t("errors.no_enhanced"));
    }
  }
}

async function generateSample() {
  if (!selectedFilePath) {
    showError(t("errors.no_file"));
    return;
  }

  // Generate output path
  const inputPath = selectedFilePath;
  const outputPath = inputPath.replace(/\.[^.]+$/, "_enhanced_sample.mp4");

  try {
    currentProcessingPath = outputPath;
    isProcessing = true;
    showProgress(t("progress.status_generating"));

    // Call Tauri command
    const result = await invoke<string>("generate_sample", {
      inputPath: inputPath,
      outputPath: outputPath
    });

    console.log("Sample generated at:", result);

    isProcessing = false;
    currentProcessingPath = null;
    hideProgress();
    enhancedVideoPath = result;

    // Load enhanced video using convertFileSrc
    const assetUrl = await convertFileSrc(result);
    enhancedVideoObjectUrl = assetUrl;

    console.log("Loading enhanced video from:", assetUrl);

    // Set up load handlers before setting src
    await new Promise<void>((resolve, reject) => {
      enhancedVideo.onloadeddata = () => {
        console.log("Enhanced video loaded successfully!");
        resolve();
      };

      enhancedVideo.onerror = (e) => {
        console.error("Enhanced video load error:", e);
        console.error("Video error code:", enhancedVideo.error?.code);
        console.error("Video error message:", enhancedVideo.error?.message);
        reject(new Error(`Video load failed: ${enhancedVideo.error?.message}`));
      };

      enhancedVideo.src = assetUrl;
      enhancedVideo.load();

      // Add timeout to reject if video takes too long
      setTimeout(() => {
        if (enhancedVideo.readyState < 2) {
          reject(new Error("Video load timeout"));
        }
      }, 30000);
    });

    // Show the enhanced tab and switch to it
    enhancedTabBtn.style.display = "block";
    switchTab("enhanced");

    // Show success message with file path
    showSuccess(t("success.sample_generated", { path: result }), result, true);
  } catch (error) {
    console.error("Failed to generate sample:", error);
    isProcessing = false;
    currentProcessingPath = null;
    hideProgress();
    showError(t("errors.generate_failed", { error: String(error) }));
  }
}

async function processFullVideo() {
  if (!selectedFilePath) {
    showError(t("errors.no_file"));
    return;
  }

  const scale = parseInt((document.getElementById("scale-select") as HTMLSelectElement).value);
  const quality = (document.getElementById("quality-select") as HTMLSelectElement).value;
  const encoder = (document.getElementById("encoder-select") as HTMLSelectElement).value;

  // Generate output path
  const inputPath = selectedFilePath;
  const outputPath = inputPath.replace(/\.[^.]+$/, "_enhanced.mp4");

  try {
    currentProcessingPath = outputPath;
    isProcessing = true;
    showProgress(t("progress.status_processing"));

    // Call Tauri command
    const result = await invoke<string>("process_video", {
      inputPath: inputPath,
      outputPath: outputPath,
      scale: scale,
      quality: quality,
      hardwareEncoder: encoder
    });

    console.log("Full video processed at:", result);

    isProcessing = false;
    currentProcessingPath = null;
    hideProgress();
    enhancedVideoPath = result;

    // Load enhanced video using convertFileSrc
    const assetUrl = await convertFileSrc(result);
    enhancedVideoObjectUrl = assetUrl;

    console.log("Loading enhanced video from:", assetUrl);

    // Set up load handlers before setting src
    await new Promise<void>((resolve, reject) => {
      enhancedVideo.onloadeddata = () => {
        console.log("Enhanced video loaded successfully!");
        resolve();
      };

      enhancedVideo.onerror = (e) => {
        console.error("Enhanced video load error:", e);
        console.error("Video error code:", enhancedVideo.error?.code);
        console.error("Video error message:", enhancedVideo.error?.message);
        reject(new Error(`Video load failed: ${enhancedVideo.error?.message}`));
      };

      enhancedVideo.src = assetUrl;
      enhancedVideo.load();

      // Add timeout to reject if video takes too long
      setTimeout(() => {
        if (enhancedVideo.readyState < 2) {
          reject(new Error("Video load timeout"));
        }
      }, 30000);
    });

    // Show the enhanced tab and switch to it
    enhancedTabBtn.style.display = "block";
    switchTab("enhanced");

    // Show success message with file path
    showSuccess(t("success.full_video_generated", { path: result }), result, false);
  } catch (error) {
    console.error("Failed to process full video:", error);
    isProcessing = false;
    currentProcessingPath = null;
    hideProgress();
    showError(t("errors.process_failed", { error: String(error) }));
  }
}

function showProgress(message: string) {
  errorSection.style.display = "none";
  progressSection.style.display = "block";
  statusText.textContent = message;
  progressFill.style.width = "0%";
  progressText.textContent = "0% (0/0 frames)";

  // Disable process buttons during processing
  generateSampleBtn.disabled = true;
  processFullBtn.disabled = true;
  selectVideoBtn.disabled = true;
}

// Update progress with real data from FFmpeg
function updateProgress(current: number, total: number, percentage: number) {
  progressFill.style.width = `${percentage.toFixed(1)}%`;
  progressText.textContent = `${percentage.toFixed(1)}% (${current}/${total} frames)`;
}

function hideProgress() {
  // Show 100% complete
  progressFill.style.width = "100%";
  progressText.textContent = "100% (Complete!)";

  // Re-enable buttons
  generateSampleBtn.disabled = false;
  processFullBtn.disabled = false;
  selectVideoBtn.disabled = false;

  setTimeout(() => {
    progressSection.style.display = "none";
  }, 1500);
}

function showError(message: string) {
  errorSection.style.display = "block";
  errorText.textContent = message;
  progressSection.style.display = "none";
  successSection.style.display = "none";

  // Re-enable buttons on error
  generateSampleBtn.disabled = false;
  processFullBtn.disabled = false;
  selectVideoBtn.disabled = false;
}

function showSuccess(message: string, filePath: string, isSample: boolean = false) {
  successSection.style.display = "block";
  successMessage.textContent = message;
  successFilePath.textContent = filePath;
  progressSection.style.display = "none";
  errorSection.style.display = "none";

  // Track sample files for cleanup
  if (isSample && !sampleFiles.includes(filePath)) {
    sampleFiles.push(filePath);
  }

  // Re-enable buttons
  generateSampleBtn.disabled = false;
  processFullBtn.disabled = false;
  selectVideoBtn.disabled = false;
}

async function cancelProcessing() {
  if (!isProcessing) {
    return;
  }

  try {
    // First, cancel the FFmpeg process
    await invoke("cancel_video_processing");
    console.log("FFmpeg process cancelled");

    // Then delete the partial output file if it exists
    if (currentProcessingPath) {
      await invoke("delete_file", { path: currentProcessingPath });
      console.log("Deleted partial file:", currentProcessingPath);
    }

    // Reset state
    isProcessing = false;
    currentProcessingPath = null;

    // Hide progress and re-enable buttons
    progressSection.style.display = "none";
    generateSampleBtn.disabled = false;
    processFullBtn.disabled = false;
    selectVideoBtn.disabled = false;

    console.log("Processing cancelled");
  } catch (error) {
    console.error("Error cancelling:", error);
    showError(t("errors.cancel_failed", { error: String(error) }));
    // Still re-enable buttons on error
    isProcessing = false;
    currentProcessingPath = null;
    generateSampleBtn.disabled = false;
    processFullBtn.disabled = false;
    selectVideoBtn.disabled = false;
  }
}
