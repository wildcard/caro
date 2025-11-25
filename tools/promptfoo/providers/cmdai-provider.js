/**
 * Custom promptfoo provider for cmdai binary
 *
 * This provider wraps the cmdai CLI binary and executes it via child process,
 * capturing the generated command output for evaluation.
 */

import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Execute cmdai binary and capture output
 *
 * @param {string} prompt - The user's natural language request
 * @param {object} context - Promptfoo context with config
 * @returns {Promise<object>} Provider response with output
 */
export default async function callCmdai(prompt, context) {
  const config = context.provider?.config || {};

  // Resolve cmdai binary path (relative to this file)
  const binaryPath = config.binaryPath || '../../../target/release/cmdai';
  const resolvedPath = path.resolve(__dirname, binaryPath);

  const shell = config.shell || 'bash';
  const timeout = config.timeout || 10000; // 10 seconds default

  // Log execution for debugging
  console.log(`[cmdai-provider] Executing: ${resolvedPath} "${prompt}"`);

  return new Promise((resolve, reject) => {
    let stdout = '';
    let stderr = '';
    let timedOut = false;

    // Spawn cmdai process
    const child = spawn(resolvedPath, [prompt, '--shell', shell], {
      timeout: timeout,
      env: {
        ...process.env,
        RUST_LOG: 'error', // Suppress debug logs
      }
    });

    // Set up timeout
    const timer = setTimeout(() => {
      timedOut = true;
      child.kill('SIGTERM');
      reject(new Error(`cmdai execution timed out after ${timeout}ms`));
    }, timeout);

    // Capture stdout
    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    // Capture stderr
    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    // Handle process exit
    child.on('close', (code) => {
      clearTimeout(timer);

      if (timedOut) {
        return; // Already rejected
      }

      // Extract command from output
      const command = extractCommand(stdout, stderr);

      if (code === 0 && command) {
        console.log(`[cmdai-provider] Success: ${command}`);
        resolve({
          output: command,
          metadata: {
            exitCode: code,
            stderr: stderr.trim(),
            rawOutput: stdout.trim(),
          }
        });
      } else {
        console.error(`[cmdai-provider] Error: Exit code ${code}`);
        console.error(`[cmdai-provider] stderr: ${stderr}`);

        // Return error information but don't reject
        // This allows promptfoo to evaluate failures
        resolve({
          output: stderr || `Command failed with exit code ${code}`,
          error: `cmdai exited with code ${code}`,
          metadata: {
            exitCode: code,
            stderr: stderr.trim(),
            rawOutput: stdout.trim(),
          }
        });
      }
    });

    // Handle spawn errors
    child.on('error', (error) => {
      clearTimeout(timer);
      console.error(`[cmdai-provider] Spawn error: ${error.message}`);

      reject(new Error(`Failed to spawn cmdai: ${error.message}`));
    });
  });
}

/**
 * Extract the generated command from cmdai output
 *
 * cmdai may output in various formats:
 * 1. Plain command
 * 2. "Command: <cmd>"
 * 3. JSON format: {"cmd": "..."}
 * 4. Multi-line with explanation
 *
 * @param {string} stdout - Standard output from cmdai
 * @param {string} stderr - Standard error from cmdai
 * @returns {string|null} Extracted command or null
 */
function extractCommand(stdout, stderr) {
  if (!stdout || stdout.trim().length === 0) {
    return null;
  }

  const lines = stdout.split('\n').map(line => line.trim()).filter(line => line.length > 0);

  // Strategy 1: Look for "Command:" prefix
  for (const line of lines) {
    if (line.startsWith('Command:')) {
      const cmd = line.substring('Command:'.length).trim();
      if (cmd.length > 0) {
        return cmd;
      }
    }
  }

  // Strategy 2: Try to parse as JSON
  try {
    const json = JSON.parse(stdout);
    if (json.cmd) {
      return json.cmd;
    }
    if (json.command) {
      return json.command;
    }
  } catch (e) {
    // Not JSON, continue
  }

  // Strategy 3: Look for lines that start with common command prefixes
  const commandPrefixes = [
    'find', 'ls', 'grep', 'awk', 'sed', 'sort', 'uniq', 'wc', 'cat', 'head',
    'tail', 'du', 'df', 'ps', 'top', 'tar', 'gzip', 'zip', 'chmod', 'chown'
  ];

  for (const line of lines) {
    // Skip lines that look like log messages
    if (line.includes('INFO') || line.includes('DEBUG') || line.includes('ERROR')) {
      continue;
    }

    // Check if line starts with a known command
    for (const prefix of commandPrefixes) {
      if (line.startsWith(prefix + ' ') || line === prefix) {
        return line;
      }
    }
  }

  // Strategy 4: Return the first non-empty line that doesn't look like a log
  for (const line of lines) {
    if (!line.includes('INFO') &&
        !line.includes('DEBUG') &&
        !line.includes('ERROR') &&
        !line.includes('Generated command:') &&
        !line.includes('Explanation:')) {
      return line;
    }
  }

  // Strategy 5: Last resort - return the whole output
  return stdout.trim();
}

/**
 * Export provider metadata for promptfoo
 */
export const metadata = {
  id: 'cmdai-provider',
  label: 'cmdai Binary Provider',
  description: 'Custom provider that executes cmdai binary for command generation',
  version: '0.1.0',
};
