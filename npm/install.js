#!/usr/bin/env node
/**
 * Post-install script that downloads the appropriate caro binary
 * for the current platform from GitHub Releases.
 */

const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const PACKAGE_VERSION = require('./package.json').version;
const GITHUB_REPO = 'wildcard/caro';

// Platform mappings
const PLATFORM_MAP = {
  'darwin-x64': 'caro-macos-intel',
  'darwin-arm64': 'caro-macos-silicon',
  'linux-x64': 'caro-linux-amd64',
  'linux-arm64': 'caro-linux-arm64',
  'win32-x64': 'caro-windows-amd64.exe'
};

function getPlatformKey() {
  const platform = process.platform;
  const arch = process.arch;
  return `${platform}-${arch}`;
}

function getAssetName() {
  const key = getPlatformKey();
  const asset = PLATFORM_MAP[key];
  if (!asset) {
    throw new Error(`Unsupported platform: ${key}. Supported platforms: ${Object.keys(PLATFORM_MAP).join(', ')}`);
  }
  return asset;
}

function downloadFile(url, destPath) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(destPath);

    const request = (urlToFetch) => {
      https.get(urlToFetch, (response) => {
        // Handle redirects
        if (response.statusCode === 302 || response.statusCode === 301) {
          request(response.headers.location);
          return;
        }

        if (response.statusCode !== 200) {
          reject(new Error(`Failed to download: HTTP ${response.statusCode}`));
          return;
        }

        response.pipe(file);
        file.on('finish', () => {
          file.close(resolve);
        });
      }).on('error', (err) => {
        fs.unlink(destPath, () => {});
        reject(err);
      });
    };

    request(url);
  });
}

async function main() {
  const binDir = path.join(__dirname, 'bin');
  const assetName = getAssetName();
  const isWindows = process.platform === 'win32';
  const binaryName = isWindows ? 'caro.exe' : 'caro';
  const binaryPath = path.join(binDir, binaryName);

  // Create bin directory
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  // Download URL from GitHub Releases
  const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/download/v${PACKAGE_VERSION}/${assetName}`;

  console.log(`Downloading caro v${PACKAGE_VERSION} for ${getPlatformKey()}...`);
  console.log(`URL: ${downloadUrl}`);

  try {
    await downloadFile(downloadUrl, binaryPath);

    // Make executable on Unix platforms
    if (!isWindows) {
      fs.chmodSync(binaryPath, 0o755);
    }

    console.log(`Successfully installed caro to ${binaryPath}`);

    // Verify the binary works
    try {
      const version = execSync(`"${binaryPath}" --version`, { encoding: 'utf8' }).trim();
      console.log(`Verified: ${version}`);
    } catch (e) {
      console.warn('Warning: Could not verify binary. You may need to install additional dependencies.');
    }
  } catch (error) {
    console.error(`Failed to install caro: ${error.message}`);
    console.error('');
    console.error('You can install caro manually:');
    console.error(`  curl -fsSL https://github.com/${GITHUB_REPO}/releases/download/v${PACKAGE_VERSION}/${assetName} -o ${binaryPath}`);
    process.exit(1);
  }
}

main();
