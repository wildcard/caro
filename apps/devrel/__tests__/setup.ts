/**
 * Vitest test setup
 * Runs before all tests
 */

import '@testing-library/jest-dom';
import { afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';

// Cleanup after each test
afterEach(() => {
  cleanup();
});

// Mock window.matchMedia (not available in jsdom)
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {}, // deprecated
    removeListener: () => {}, // deprecated
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  }),
});

// Mock localStorage (with encryption support for testing)
const localStorageMock = (() => {
  let store: Record<string, string> = {};

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value.toString();
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
    get length() {
      return Object.keys(store).length;
    },
    key: (index: number) => {
      const keys = Object.keys(store);
      return keys[index] || null;
    },
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

// Mock IndexedDB (for future use)
// We'll use fake-indexeddb once needed for privacy storage tests

// Mock crypto.subtle (for encryption tests)
if (!globalThis.crypto) {
  (globalThis as any).crypto = {
    subtle: {
      digest: async (algorithm: string, data: BufferSource) => {
        // Simple mock hash for testing
        const buffer = new Uint8Array(32);
        return buffer.buffer;
      },
      encrypt: async () => new ArrayBuffer(0),
      decrypt: async () => new ArrayBuffer(0),
      generateKey: async () => ({}) as CryptoKey,
      importKey: async () => ({}) as CryptoKey,
      exportKey: async () => new ArrayBuffer(0),
      deriveBits: async () => new ArrayBuffer(0),
      deriveKey: async () => ({}) as CryptoKey,
      sign: async () => new ArrayBuffer(0),
      verify: async () => false,
      wrapKey: async () => new ArrayBuffer(0),
      unwrapKey: async () => ({}) as CryptoKey,
    },
    getRandomValues: (array: Uint8Array) => {
      for (let i = 0; i < array.length; i++) {
        array[i] = Math.floor(Math.random() * 256);
      }
      return array;
    },
  };
}
