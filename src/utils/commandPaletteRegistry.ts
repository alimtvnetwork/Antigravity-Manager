/**
 * Global Command Palette Registry
 * Stores, registers, and searches navigation, instance, database, and system commands with fuzzy query matching.
 */

export type CommandCategory = 'navigation' | 'instance' | 'database' | 'account' | 'proxy' | 'system';

export interface CommandItem {
  id: string;
  title: string;
  subtitle?: string;
  category: CommandCategory;
  shortcut?: string;
  action: () => void | Promise<void>;
  iconName?: string;
}

class CommandRegistry {
  private commands = new Map<string, CommandItem>();

  register(command: CommandItem): () => void {
    this.commands.set(command.id, command);
    return () => this.commands.delete(command.id);
  }

  unregister(commandId: string): void {
    this.commands.delete(commandId);
  }

  getAll(): CommandItem[] {
    return Array.from(this.commands.values());
  }

  search(query: string): CommandItem[] {
    const q = query.trim().toLowerCase();
    if (!q) {
      return this.getAll();
    }

    const items = this.getAll();
    return items
      .map(item => {
        let score = 0;
        const titleLower = item.title.toLowerCase();
        const subLower = (item.subtitle || '').toLowerCase();
        const catLower = item.category.toLowerCase();

        if (titleLower === q) score += 100;
        else if (titleLower.startsWith(q)) score += 60;
        else if (titleLower.includes(q)) score += 40;

        if (subLower.includes(q)) score += 20;
        if (catLower.includes(q)) score += 10;

        return { item, score };
      })
      .filter(entry => entry.score > 0)
      .sort((a, b) => b.score - a.score)
      .map(entry => entry.item);
  }
}

export const commandRegistry = new CommandRegistry();

export function registerDefaultSystemCommands(navigate: (path: string) => void): void {
  commandRegistry.register({
    id: 'nav.instances',
    title: 'Go to Instances',
    subtitle: 'View and manage running IDE instances and profiles',
    category: 'navigation',
    shortcut: 'Ctrl+1',
    action: () => navigate('/instances'),
  });

  commandRegistry.register({
    id: 'nav.accounts',
    title: 'Go to Accounts',
    subtitle: 'Manage Google OAuth accounts, tokens, and quota scores',
    category: 'navigation',
    shortcut: 'Ctrl+2',
    action: () => navigate('/accounts'),
  });

  commandRegistry.register({
    id: 'nav.database',
    title: 'Open Split-DB Explorer',
    subtitle: 'Browse and inspect SQLite tables, schemas, and statistics',
    category: 'database',
    shortcut: 'Ctrl+3',
    action: () => navigate('/database'),
  });

  commandRegistry.register({
    id: 'nav.settings',
    title: 'Open Settings',
    subtitle: 'Configure auto-switcher, upstream proxies, and notifications',
    category: 'system',
    shortcut: 'Ctrl+,',
    action: () => navigate('/settings'),
  });
}
