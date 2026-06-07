#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const ROBOST_CORE_DIR = path.join(__dirname, '../../..');
const RPA_SOURCE = path.join(ROBOST_CORE_DIR, 'crates/robost-cli');
const ASSETS_DIR = path.join(__dirname, '../assets/rpa');

console.log('🔨 Building robost-cli for distribution...\n');

const targets = [
  {
    name: 'macOS ARM64',
    target: 'aarch64-apple-darwin',
    dest: path.join(ASSETS_DIR, 'darwin-arm64/rpa'),
  },
  {
    name: 'macOS x64',
    target: 'x86_64-apple-darwin',
    dest: path.join(ASSETS_DIR, 'darwin-x64/rpa'),
  },
  {
    name: 'Windows x64',
    target: 'x86_64-pc-windows-gnu',
    dest: path.join(ASSETS_DIR, 'win32-x64/rpa.exe'),
  },
];

function buildTarget(name, target, dest) {
  try {
    console.log(`📦 Building ${name}...`);
    const args = ['build', '--release', '--target', target];
    execFileSync('cargo', args, {
      cwd: RPA_SOURCE,
      stdio: 'inherit',
    });

    // Copy binary to assets
    const binaryName = target.includes('windows') ? 'rpa.exe' : 'rpa';
    const sourceBinary = path.join(
      RPA_SOURCE,
      `target/${target}/release/${binaryName}`
    );

    if (!fs.existsSync(sourceBinary)) {
      console.error(`✗ Binary not found: ${sourceBinary}`);
      return false;
    }

    const destDir = path.dirname(dest);
    if (!fs.existsSync(destDir)) {
      fs.mkdirSync(destDir, { recursive: true });
    }

    fs.copyFileSync(sourceBinary, dest);
    fs.chmodSync(dest, 0o755);
    console.log(`✓ ${name}: ${dest}\n`);
    return true;
  } catch (err) {
    console.error(`✗ Failed to build ${name}:`, err.message);
    return false;
  }
}

let allSuccess = true;
for (const { name, target, dest } of targets) {
  // Check if cross is available for cross-compilation
  const isCrossTarget = process.platform !== 'darwin' && target.includes('darwin');
  const isWindowsTarget = target.includes('windows');

  if (isCrossTarget || isWindowsTarget) {
    console.log(`⚠️  Skipping ${name} (requires ${target} toolchain on current platform)\n`);
    continue;
  }

  if (!buildTarget(name, target, dest)) {
    allSuccess = false;
  }
}

if (allSuccess) {
  console.log('✓ All rpa binaries built successfully!');
  process.exit(0);
} else {
  console.error('✗ Some builds failed. See above for details.');
  process.exit(1);
}
