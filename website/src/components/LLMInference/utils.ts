/**
 * Terminal Tutor Utilities
 * Helper functions for terminal simulation and learning experience
 */

import type {
  VirtualFileSystem,
  VirtualDirectory,
  VirtualFile,
  TerminalState,
  TerminalHistoryEntry,
  TerminalCommand,
  TerminalLesson,
  LearningProgress,
} from './types';

/**
 * Create the initial virtual file system for learning
 */
export function createVirtualFileSystem(): VirtualFileSystem {
  const now = new Date();

  return {
    root: {
      name: '/',
      type: 'directory',
      permissions: 'drwxr-xr-x',
      owner: 'root',
      modified: now,
      children: [
        {
          name: 'home',
          type: 'directory',
          permissions: 'drwxr-xr-x',
          owner: 'root',
          modified: now,
          children: [
            {
              name: 'learner',
              type: 'directory',
              permissions: 'drwxr-xr-x',
              owner: 'learner',
              modified: now,
              children: [
                {
                  name: 'Documents',
                  type: 'directory',
                  permissions: 'drwxr-xr-x',
                  owner: 'learner',
                  modified: now,
                  children: [
                    {
                      name: 'notes.txt',
                      type: 'file',
                      content: 'Welcome to the terminal tutor!\nThis is your first text file.\nFeel free to explore.',
                      size: 85,
                      permissions: '-rw-r--r--',
                      owner: 'learner',
                      modified: now,
                    },
                    {
                      name: 'todo.txt',
                      type: 'file',
                      content: '1. Learn basic navigation\n2. Master file operations\n3. Understand pipes',
                      size: 67,
                      permissions: '-rw-r--r--',
                      owner: 'learner',
                      modified: now,
                    },
                  ],
                },
                {
                  name: 'Projects',
                  type: 'directory',
                  permissions: 'drwxr-xr-x',
                  owner: 'learner',
                  modified: now,
                  children: [
                    {
                      name: 'hello-world',
                      type: 'directory',
                      permissions: 'drwxr-xr-x',
                      owner: 'learner',
                      modified: now,
                      children: [
                        {
                          name: 'README.md',
                          type: 'file',
                          content: '# Hello World Project\n\nA simple project to practice terminal commands.',
                          size: 62,
                          permissions: '-rw-r--r--',
                          owner: 'learner',
                          modified: now,
                        },
                        {
                          name: 'script.sh',
                          type: 'file',
                          content: '#!/bin/bash\necho "Hello, Terminal Learner!"',
                          size: 44,
                          permissions: '-rwxr-xr-x',
                          owner: 'learner',
                          modified: now,
                        },
                      ],
                    },
                  ],
                },
                {
                  name: '.bashrc',
                  type: 'file',
                  content: '# Bash configuration\nexport PS1="\\u@\\h:\\w$ "\nalias ll="ls -la"',
                  size: 72,
                  permissions: '-rw-r--r--',
                  owner: 'learner',
                  modified: now,
                },
                {
                  name: '.hidden_treasure',
                  type: 'file',
                  content: 'Congratulations! You found the hidden file using ls -a!',
                  size: 54,
                  permissions: '-rw-r--r--',
                  owner: 'learner',
                  modified: now,
                },
              ],
            },
          ],
        },
        {
          name: 'tmp',
          type: 'directory',
          permissions: 'drwxrwxrwt',
          owner: 'root',
          modified: now,
          children: [
            {
              name: 'scratch.txt',
              type: 'file',
              content: 'Temporary file for practice',
              size: 27,
              permissions: '-rw-r--r--',
              owner: 'learner',
              modified: now,
            },
          ],
        },
      ],
    },
  };
}

/**
 * Create initial terminal state
 */
export function createInitialTerminalState(): TerminalState {
  return {
    currentDirectory: '/home/learner',
    history: [],
    environment: {
      HOME: '/home/learner',
      USER: 'learner',
      PATH: '/usr/local/bin:/usr/bin:/bin',
      SHELL: '/bin/bash',
    },
    user: 'learner',
    hostname: 'caro-tutor',
  };
}

/**
 * Resolve a path relative to current directory
 */
export function resolvePath(currentDir: string, targetPath: string): string {
  if (targetPath.startsWith('/')) {
    return normalizePath(targetPath);
  }

  if (targetPath === '~') {
    return '/home/learner';
  }

  if (targetPath.startsWith('~/')) {
    return '/home/learner' + targetPath.slice(1);
  }

  const parts = currentDir.split('/').filter(Boolean);

  for (const segment of targetPath.split('/')) {
    if (segment === '..') {
      parts.pop();
    } else if (segment !== '.' && segment !== '') {
      parts.push(segment);
    }
  }

  return '/' + parts.join('/');
}

/**
 * Normalize a file path
 */
export function normalizePath(path: string): string {
  const parts = path.split('/').filter(Boolean);
  const result: string[] = [];

  for (const part of parts) {
    if (part === '..') {
      result.pop();
    } else if (part !== '.') {
      result.push(part);
    }
  }

  return '/' + result.join('/');
}

/**
 * Get a node from the virtual file system by path
 */
export function getNodeAtPath(
  fs: VirtualFileSystem,
  path: string
): VirtualDirectory | VirtualFile | null {
  const normalizedPath = normalizePath(path);

  if (normalizedPath === '/') {
    return fs.root;
  }

  const parts = normalizedPath.split('/').filter(Boolean);
  let current: VirtualDirectory | VirtualFile = fs.root;

  for (const part of parts) {
    if (current.type !== 'directory') {
      return null;
    }

    const found = current.children.find((child) => child.name === part);
    if (!found) {
      return null;
    }

    current = found;
  }

  return current;
}

/**
 * Format file listing output
 */
export function formatLsOutput(
  items: (VirtualDirectory | VirtualFile)[],
  options: { showAll?: boolean; longFormat?: boolean } = {}
): string {
  const { showAll = false, longFormat = false } = options;

  // Filter hidden files unless -a is used
  const filteredItems = showAll
    ? items
    : items.filter((item) => !item.name.startsWith('.'));

  if (filteredItems.length === 0) {
    return '';
  }

  if (longFormat) {
    const lines = filteredItems.map((item) => {
      const size = item.type === 'file' ? item.size.toString().padStart(6) : '  4096';
      const date = formatDate(item.modified);
      const name = item.type === 'directory' ? `\x1b[34m${item.name}/\x1b[0m` : item.name;
      return `${item.permissions} ${item.owner.padEnd(8)} ${size} ${date} ${name}`;
    });
    return lines.join('\n');
  }

  return filteredItems
    .map((item) => (item.type === 'directory' ? `\x1b[34m${item.name}/\x1b[0m` : item.name))
    .join('  ');
}

/**
 * Format date for ls -l output
 */
function formatDate(date: Date): string {
  const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  const month = months[date.getMonth()];
  const day = date.getDate().toString().padStart(2);
  const time = `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  return `${month} ${day} ${time}`;
}

/**
 * Terminal command definitions for learning
 */
export const TERMINAL_COMMANDS: TerminalCommand[] = [
  {
    id: 'pwd',
    command: 'pwd',
    description: 'Print the current working directory path',
    category: 'navigation',
    difficulty: 'beginner',
    safetyLevel: 'safe',
    examples: [
      {
        input: 'pwd',
        output: '/home/learner',
        explanation: 'Shows you exactly where you are in the file system',
      },
    ],
    relatedCommands: ['cd', 'ls'],
  },
  {
    id: 'ls',
    command: 'ls',
    description: 'List directory contents',
    category: 'navigation',
    difficulty: 'beginner',
    safetyLevel: 'safe',
    examples: [
      {
        input: 'ls',
        output: 'Documents  Projects',
        explanation: 'Shows files and folders in the current directory',
      },
      {
        input: 'ls -a',
        output: '.  ..  .bashrc  .hidden_treasure  Documents  Projects',
        explanation: 'Shows ALL files including hidden ones (starting with .)',
      },
      {
        input: 'ls -l',
        output: 'drwxr-xr-x learner 4096 Dec 23 10:00 Documents/',
        explanation: 'Shows detailed information about each file',
      },
    ],
    relatedCommands: ['pwd', 'cd'],
  },
  {
    id: 'cd',
    command: 'cd',
    description: 'Change the current directory',
    category: 'navigation',
    difficulty: 'beginner',
    safetyLevel: 'safe',
    examples: [
      {
        input: 'cd Documents',
        output: '',
        explanation: 'Moves into the Documents folder',
      },
      {
        input: 'cd ..',
        output: '',
        explanation: 'Moves up one directory level',
      },
      {
        input: 'cd ~',
        output: '',
        explanation: 'Returns to your home directory',
      },
    ],
    relatedCommands: ['pwd', 'ls'],
  },
  {
    id: 'cat',
    command: 'cat',
    description: 'Display file contents',
    category: 'viewing',
    difficulty: 'beginner',
    safetyLevel: 'safe',
    examples: [
      {
        input: 'cat notes.txt',
        output: 'Welcome to the terminal tutor!',
        explanation: 'Shows the entire contents of a file',
      },
    ],
    relatedCommands: ['head', 'tail', 'less'],
  },
  {
    id: 'mkdir',
    command: 'mkdir',
    description: 'Create a new directory',
    category: 'files',
    difficulty: 'beginner',
    safetyLevel: 'safe',
    examples: [
      {
        input: 'mkdir my-project',
        output: '',
        explanation: 'Creates a new folder called my-project',
      },
    ],
    relatedCommands: ['rmdir', 'touch'],
  },
  {
    id: 'touch',
    command: 'touch',
    description: 'Create an empty file',
    category: 'files',
    difficulty: 'beginner',
    safetyLevel: 'safe',
    examples: [
      {
        input: 'touch newfile.txt',
        output: '',
        explanation: 'Creates an empty file called newfile.txt',
      },
    ],
    relatedCommands: ['mkdir', 'rm'],
  },
  {
    id: 'cp',
    command: 'cp',
    description: 'Copy files or directories',
    category: 'files',
    difficulty: 'intermediate',
    safetyLevel: 'moderate',
    examples: [
      {
        input: 'cp file.txt backup.txt',
        output: '',
        explanation: 'Makes a copy of file.txt named backup.txt',
      },
      {
        input: 'cp -r folder/ backup/',
        output: '',
        explanation: 'Recursively copies an entire directory',
      },
    ],
    relatedCommands: ['mv', 'rm'],
  },
  {
    id: 'mv',
    command: 'mv',
    description: 'Move or rename files',
    category: 'files',
    difficulty: 'intermediate',
    safetyLevel: 'moderate',
    examples: [
      {
        input: 'mv old.txt new.txt',
        output: '',
        explanation: 'Renames old.txt to new.txt',
      },
      {
        input: 'mv file.txt Documents/',
        output: '',
        explanation: 'Moves file.txt into the Documents folder',
      },
    ],
    relatedCommands: ['cp', 'rm'],
  },
  {
    id: 'rm',
    command: 'rm',
    description: 'Remove files (use with caution!)',
    category: 'files',
    difficulty: 'intermediate',
    safetyLevel: 'dangerous',
    examples: [
      {
        input: 'rm temp.txt',
        output: '',
        explanation: 'Permanently deletes temp.txt - no undo!',
      },
    ],
    relatedCommands: ['rmdir', 'mv'],
  },
  {
    id: 'head',
    command: 'head',
    description: 'Show the first lines of a file',
    category: 'viewing',
    difficulty: 'intermediate',
    safetyLevel: 'safe',
    examples: [
      {
        input: 'head -n 5 file.txt',
        output: '(first 5 lines)',
        explanation: 'Shows only the first 5 lines of a file',
      },
    ],
    relatedCommands: ['tail', 'cat'],
  },
  {
    id: 'tail',
    command: 'tail',
    description: 'Show the last lines of a file',
    category: 'viewing',
    difficulty: 'intermediate',
    safetyLevel: 'safe',
    examples: [
      {
        input: 'tail -n 5 file.txt',
        output: '(last 5 lines)',
        explanation: 'Shows only the last 5 lines of a file',
      },
    ],
    relatedCommands: ['head', 'cat'],
  },
  {
    id: 'grep',
    command: 'grep',
    description: 'Search for patterns in files',
    category: 'advanced',
    difficulty: 'advanced',
    safetyLevel: 'safe',
    examples: [
      {
        input: 'grep "error" log.txt',
        output: 'Lines containing "error"',
        explanation: 'Finds all lines containing the word "error"',
      },
    ],
    relatedCommands: ['find', 'cat'],
  },
];

/**
 * Learning lessons for progressive education
 */
export const TERMINAL_LESSONS: TerminalLesson[] = [
  {
    id: 'lesson-1-orientation',
    title: 'Getting Oriented',
    description: 'Learn where you are and how to look around',
    level: 1,
    objectives: [
      'Understand what a working directory is',
      'Learn to check your current location',
      'Learn to list files and folders',
    ],
    commands: ['pwd', 'ls'],
    exercises: [
      {
        id: 'ex-1-1',
        prompt: 'First, let\'s find out where you are. Type "pwd" to print your current directory.',
        expectedCommand: 'pwd',
        hints: ['pwd stands for "print working directory"'],
        successMessage: 'Great! You\'re in /home/learner - this is your home directory.',
        explanation: 'The pwd command shows your current location in the file system.',
      },
      {
        id: 'ex-1-2',
        prompt: 'Now let\'s see what\'s here. Type "ls" to list the contents.',
        expectedCommand: 'ls',
        hints: ['ls is short for "list"'],
        successMessage: 'You can see the Documents and Projects folders!',
        explanation: 'The ls command lists files and directories in your current location.',
      },
      {
        id: 'ex-1-3',
        prompt: 'Some files are hidden! Use "ls -a" to see ALL files.',
        expectedCommand: 'ls -a',
        hints: ['The -a flag means "all"', 'Hidden files start with a dot (.)'],
        successMessage: 'You found hidden files like .bashrc and .hidden_treasure!',
        explanation: 'Hidden files in Unix start with a dot. The -a flag reveals them.',
      },
    ],
    completionCriteria: ['Used pwd', 'Used ls', 'Used ls -a'],
  },
  {
    id: 'lesson-2-navigation',
    title: 'Moving Around',
    description: 'Learn to navigate the file system',
    level: 2,
    objectives: [
      'Change directories with cd',
      'Navigate up and down the folder tree',
      'Return to home directory',
    ],
    commands: ['cd'],
    exercises: [
      {
        id: 'ex-2-1',
        prompt: 'Let\'s go into the Documents folder. Type "cd Documents".',
        expectedCommand: 'cd Documents',
        hints: ['cd stands for "change directory"'],
        successMessage: 'You\'re now in the Documents folder!',
        explanation: 'The cd command changes your current directory.',
      },
      {
        id: 'ex-2-2',
        prompt: 'Now go back up one level with "cd .."',
        expectedCommand: 'cd ..',
        hints: ['.. means "parent directory"'],
        successMessage: 'You moved back to your home directory!',
        explanation: 'Two dots (..) always refer to the parent directory.',
      },
      {
        id: 'ex-2-3',
        prompt: 'Try going deeper: "cd Projects/hello-world"',
        expectedCommand: 'cd Projects/hello-world',
        hints: ['You can navigate multiple levels at once using /'],
        successMessage: 'You navigated two levels deep in one command!',
        explanation: 'You can chain directories with / to move multiple levels.',
      },
    ],
    completionCriteria: ['Used cd to enter a folder', 'Used cd .. to go up', 'Used cd with a path'],
  },
  {
    id: 'lesson-3-viewing',
    title: 'Reading Files',
    description: 'Learn to view file contents',
    level: 3,
    objectives: [
      'Display entire file contents',
      'View just the beginning of files',
      'View just the end of files',
    ],
    commands: ['cat', 'head', 'tail'],
    exercises: [
      {
        id: 'ex-3-1',
        prompt: 'Let\'s read a file! Go to Documents and use "cat notes.txt"',
        expectedCommand: 'cat notes.txt',
        hints: ['Make sure you\'re in the Documents folder first'],
        successMessage: 'You can see the entire file contents!',
        explanation: 'cat displays the complete contents of a file.',
      },
      {
        id: 'ex-3-2',
        prompt: 'For long files, use "head" to see just the beginning.',
        expectedCommand: 'head notes.txt',
        hints: ['head shows the first 10 lines by default'],
        successMessage: 'head is great for previewing large files!',
        explanation: 'head shows only the first lines of a file.',
      },
    ],
    completionCriteria: ['Used cat to view a file', 'Used head to preview a file'],
  },
];

/**
 * Get command info by ID
 */
export function getCommandInfo(commandId: string): TerminalCommand | undefined {
  return TERMINAL_COMMANDS.find((cmd) => cmd.id === commandId);
}

/**
 * Get lesson by ID
 */
export function getLessonById(lessonId: string): TerminalLesson | undefined {
  return TERMINAL_LESSONS.find((lesson) => lesson.id === lessonId);
}

/**
 * Save learning progress to localStorage
 */
export function saveLearningProgress(progress: LearningProgress): void {
  localStorage.setItem('terminalTutorProgress', JSON.stringify(progress));
}

/**
 * Load learning progress from localStorage
 */
export function loadLearningProgress(): LearningProgress | null {
  const saved = localStorage.getItem('terminalTutorProgress');
  if (!saved) return null;

  try {
    const progress = JSON.parse(saved);
    progress.lastSession = new Date(progress.lastSession);
    return progress;
  } catch {
    return null;
  }
}

/**
 * Create initial learning progress
 */
export function createInitialProgress(): LearningProgress {
  return {
    currentLevel: 1,
    completedLessons: [],
    commandsLearned: [],
    totalScore: 0,
    streakDays: 0,
    lastSession: new Date(),
  };
}

/**
 * Format bytes to human-readable size
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';

  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

/**
 * Generate unique ID
 */
export function generateId(): string {
  return crypto.randomUUID?.() ?? Math.random().toString(36).substring(2, 11);
}

/**
 * Escape HTML for safe display
 */
export function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

/**
 * Parse ANSI color codes to HTML
 */
export function ansiToHtml(text: string): string {
  return text
    .replace(/\x1b\[34m/g, '<span class="text-blue">')
    .replace(/\x1b\[32m/g, '<span class="text-green">')
    .replace(/\x1b\[31m/g, '<span class="text-red">')
    .replace(/\x1b\[33m/g, '<span class="text-yellow">')
    .replace(/\x1b\[0m/g, '</span>');
}

/**
 * Debounce function for performance
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;

  return function (...args: Parameters<T>) {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => func(...args), wait);
  };
}
