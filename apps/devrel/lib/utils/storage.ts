/**
 * Storage utilities for localStorage and IndexedDB
 * Provides type-safe storage operations
 */

/**
 * localStorage wrapper with JSON serialization
 */
export const storage = {
  /**
   * Get item from localStorage
   */
  get: <T>(key: string): T | null => {
    if (typeof window === 'undefined') return null;

    try {
      const item = localStorage.getItem(key);
      if (!item) return null;
      return JSON.parse(item) as T;
    } catch (error) {
      console.error(`Error reading from localStorage (${key}):`, error);
      return null;
    }
  },

  /**
   * Set item in localStorage
   */
  set: <T>(key: string, value: T): boolean => {
    if (typeof window === 'undefined') return false;

    try {
      localStorage.setItem(key, JSON.stringify(value));
      return true;
    } catch (error) {
      console.error(`Error writing to localStorage (${key}):`, error);
      return false;
    }
  },

  /**
   * Remove item from localStorage
   */
  remove: (key: string): boolean => {
    if (typeof window === 'undefined') return false;

    try {
      localStorage.removeItem(key);
      return true;
    } catch (error) {
      console.error(`Error removing from localStorage (${key}):`, error);
      return false;
    }
  },

  /**
   * Clear all items from localStorage
   */
  clear: (): boolean => {
    if (typeof window === 'undefined') return false;

    try {
      localStorage.clear();
      return true;
    } catch (error) {
      console.error('Error clearing localStorage:', error);
      return false;
    }
  },

  /**
   * Get storage size in bytes (approximate)
   */
  getSize: (): number => {
    if (typeof window === 'undefined') return 0;

    let size = 0;
    for (const key in localStorage) {
      if (localStorage.hasOwnProperty(key)) {
        size += key.length + (localStorage.getItem(key)?.length || 0);
      }
    }
    return size;
  },
};

/**
 * Storage keys (centralized for type safety)
 */
export const STORAGE_KEYS = {
  // Auth
  AUTH_STATE: 'caro:auth:state',
  OAUTH_STATE: 'caro:oauth:state',

  // User
  USER_PROFILE: 'caro:user:profile',
  PRIVACY_SETTINGS: 'caro:privacy:settings',

  // Local data
  LOCAL_COMMANDS: 'caro:local:commands',
  LOCAL_RUNBOOKS: 'caro:local:runbooks',
  LOCAL_STATS: 'caro:local:stats',

  // Cache
  GUILDS_CACHE: 'caro:cache:guilds',
  FEED_CACHE: 'caro:cache:feed',

  // UI state
  ONBOARDING_COMPLETED: 'caro:ui:onboarding',
  THEME_PREFERENCE: 'caro:ui:theme',
} as const;

/**
 * IndexedDB wrapper for larger data storage
 * (To be implemented in WP03 when needed for CLI data)
 */
export const indexedDB = {
  /**
   * Initialize IndexedDB database
   */
  init: async (dbName: string, version: number): Promise<IDBDatabase | null> => {
    if (typeof window === 'undefined' || !window.indexedDB) {
      console.warn('IndexedDB not available');
      return null;
    }

    return new Promise((resolve, reject) => {
      const request = window.indexedDB.open(dbName, version);

      request.onerror = () => {
        console.error('IndexedDB error:', request.error);
        reject(request.error);
      };

      request.onsuccess = () => {
        resolve(request.result);
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        // Create object stores (tables)
        if (!db.objectStoreNames.contains('commands')) {
          db.createObjectStore('commands', { keyPath: 'id' });
        }
        if (!db.objectStoreNames.contains('runbooks')) {
          db.createObjectStore('runbooks', { keyPath: 'id' });
        }
        if (!db.objectStoreNames.contains('artifacts')) {
          db.createObjectStore('artifacts', { keyPath: 'id' });
        }
      };
    });
  },

  /**
   * Get item from IndexedDB
   */
  get: async <T>(
    db: IDBDatabase,
    storeName: string,
    key: string
  ): Promise<T | null> => {
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([storeName], 'readonly');
      const store = transaction.objectStore(storeName);
      const request = store.get(key);

      request.onsuccess = () => {
        resolve(request.result || null);
      };

      request.onerror = () => {
        console.error('IndexedDB get error:', request.error);
        reject(request.error);
      };
    });
  },

  /**
   * Set item in IndexedDB
   */
  set: async <T>(
    db: IDBDatabase,
    storeName: string,
    value: T
  ): Promise<boolean> => {
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([storeName], 'readwrite');
      const store = transaction.objectStore(storeName);
      const request = store.put(value);

      request.onsuccess = () => {
        resolve(true);
      };

      request.onerror = () => {
        console.error('IndexedDB set error:', request.error);
        reject(request.error);
      };
    });
  },

  /**
   * Delete item from IndexedDB
   */
  delete: async (
    db: IDBDatabase,
    storeName: string,
    key: string
  ): Promise<boolean> => {
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([storeName], 'readwrite');
      const store = transaction.objectStore(storeName);
      const request = store.delete(key);

      request.onsuccess = () => {
        resolve(true);
      };

      request.onerror = () => {
        console.error('IndexedDB delete error:', request.error);
        reject(request.error);
      };
    });
  },

  /**
   * Get all items from store
   */
  getAll: async <T>(
    db: IDBDatabase,
    storeName: string
  ): Promise<T[]> => {
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([storeName], 'readonly');
      const store = transaction.objectStore(storeName);
      const request = store.getAll();

      request.onsuccess = () => {
        resolve(request.result || []);
      };

      request.onerror = () => {
        console.error('IndexedDB getAll error:', request.error);
        reject(request.error);
      };
    });
  },
};
