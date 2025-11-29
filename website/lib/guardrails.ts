import fs from 'fs';
import path from 'path';
import yaml from 'js-yaml';
import type { Guardrail, GuardrailCategory, RiskLevel, GuardrailFilter } from '@/types';

const GUARDRAILS_DIR = path.join(process.cwd(), '..', 'docs', 'guardrails');

/**
 * Load all guardrails from YAML files
 */
export async function loadGuardrails(): Promise<Guardrail[]> {
  const files = fs.readdirSync(GUARDRAILS_DIR).filter(file => file.endsWith('.yml') || file.endsWith('.yaml'));

  const guardrails = files.map(file => {
    const filePath = path.join(GUARDRAILS_DIR, file);
    const fileContents = fs.readFileSync(filePath, 'utf8');
    const data = yaml.load(fileContents) as Guardrail;
    return data;
  });

  return guardrails.sort((a, b) => a.id.localeCompare(b.id));
}

/**
 * Load a single guardrail by ID
 */
export async function loadGuardrailById(id: string): Promise<Guardrail | null> {
  const guardrails = await loadGuardrails();
  return guardrails.find(g => g.id === id) || null;
}

/**
 * Filter guardrails based on criteria
 */
export function filterGuardrails(guardrails: Guardrail[], filter: GuardrailFilter): Guardrail[] {
  return guardrails.filter(g => {
    if (filter.category && g.category !== filter.category) return false;
    if (filter.risk_level && g.pattern.risk_level !== filter.risk_level) return false;
    if (filter.shell_type) {
      if (g.pattern.shell_specific && g.pattern.shell_specific !== filter.shell_type) return false;
    }
    if (filter.search) {
      const searchLower = filter.search.toLowerCase();
      const matchesSearch =
        g.pattern.description.toLowerCase().includes(searchLower) ||
        g.explanation.toLowerCase().includes(searchLower) ||
        g.tags.some(tag => tag.toLowerCase().includes(searchLower)) ||
        g.examples_blocked.some(ex => ex.toLowerCase().includes(searchLower));
      if (!matchesSearch) return false;
    }
    return true;
  });
}

/**
 * Get category metadata
 */
export function getGuardrailCategoryInfo(): Record<GuardrailCategory, { name: string; icon: string; description: string }> {
  return {
    FilesystemDestruction: {
      name: 'Filesystem Destruction',
      icon: '🗑️',
      description: 'Prevents recursive deletion and filesystem damage',
    },
    DiskOperations: {
      name: 'Disk Operations',
      icon: '💾',
      description: 'Blocks disk formatting and low-level operations',
    },
    PrivilegeEscalation: {
      name: 'Privilege Escalation',
      icon: '🔐',
      description: 'Protects against unauthorized privilege elevation',
    },
    NetworkBackdoors: {
      name: 'Network Backdoors',
      icon: '🌐',
      description: 'Prevents creation of network backdoors and shells',
    },
    ProcessManipulation: {
      name: 'Process Manipulation',
      icon: '⚙️',
      description: 'Blocks dangerous process operations',
    },
    SystemModification: {
      name: 'System Modification',
      icon: '🔧',
      description: 'Protects system configuration files',
    },
    EnvironmentManipulation: {
      name: 'Environment Manipulation',
      icon: '🔤',
      description: 'Prevents PATH hijacking and environment tampering',
    },
    PackageManagement: {
      name: 'Package Management',
      icon: '📦',
      description: 'Guards against forced package operations',
    },
    Containers: {
      name: 'Containers',
      icon: '🐳',
      description: 'Prevents container escape and privilege violations',
    },
  };
}

/**
 * Get risk level styling
 */
export function getRiskLevelColor(level: RiskLevel): string {
  switch (level) {
    case 'Safe':
      return 'text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-950';
    case 'Moderate':
      return 'text-yellow-600 dark:text-yellow-400 bg-yellow-50 dark:bg-yellow-950';
    case 'High':
      return 'text-orange-600 dark:text-orange-400 bg-orange-50 dark:bg-orange-950';
    case 'Critical':
      return 'text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-950';
  }
}

/**
 * Get guardrails by category
 */
export async function getGuardrailsByCategory(): Promise<Record<GuardrailCategory, Guardrail[]>> {
  const guardrails = await loadGuardrails();
  const byCategory: Partial<Record<GuardrailCategory, Guardrail[]>> = {};

  guardrails.forEach(g => {
    if (!byCategory[g.category]) {
      byCategory[g.category] = [];
    }
    byCategory[g.category]!.push(g);
  });

  return byCategory as Record<GuardrailCategory, Guardrail[]>;
}

/**
 * Get guardrails statistics
 */
export async function getGuardrailsStats() {
  const guardrails = await loadGuardrails();

  const total = guardrails.length;
  const byCriticality: Record<RiskLevel, number> = {
    Safe: 0,
    Moderate: 0,
    High: 0,
    Critical: 0,
  };

  const byCategory: Partial<Record<GuardrailCategory, number>> = {};

  guardrails.forEach(g => {
    byCriticality[g.pattern.risk_level]++;
    byCategory[g.category] = (byCategory[g.category] || 0) + 1;
  });

  const totalTriggers = guardrails.reduce((sum, g) => sum + g.stats.times_triggered, 0);
  const totalOverrides = guardrails.reduce((sum, g) => sum + g.stats.times_overridden, 0);

  return {
    total,
    byCriticality,
    byCategory,
    totalTriggers,
    totalOverrides,
    overrideRate: totalTriggers > 0 ? (totalOverrides / totalTriggers) * 100 : 0,
  };
}
