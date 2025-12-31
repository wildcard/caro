#!/usr/bin/env node
import { readFileSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Read Cargo.toml from the parent directory (caro root)
const cargoTomlPath = join(__dirname, '../../Cargo.toml');
const cargoToml = readFileSync(cargoTomlPath, 'utf-8');

// Extract version using a simple regex
const versionMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);

if (!versionMatch) {
  console.error('Could not find version in Cargo.toml');
  process.exit(1);
}

const version = versionMatch[1];

// Write version to a TypeScript file that can be imported
const versionFilePath = join(__dirname, '../src/config/version.ts');
const content = `// Auto-generated from Cargo.toml - DO NOT EDIT
export const CARO_VERSION = '${version}';
`;

writeFileSync(versionFilePath, content);

console.log(`✓ Extracted version ${version} from Cargo.toml`);
