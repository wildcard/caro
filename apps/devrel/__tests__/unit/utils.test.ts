/**
 * Unit tests for utility functions
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { storage, STORAGE_KEYS } from '@/lib/utils/storage';
import { generateRandomString, hash } from '@/lib/utils/crypto';

describe('Storage utilities', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('should store and retrieve JSON data', () => {
    const testData = { foo: 'bar', baz: 123 };
    const result = storage.set('test-key', testData);

    expect(result).toBe(true);
    expect(storage.get('test-key')).toEqual(testData);
  });

  it('should return null for non-existent keys', () => {
    expect(storage.get('non-existent')).toBeNull();
  });

  it('should remove items', () => {
    storage.set('test-key', { data: 'value' });
    expect(storage.get('test-key')).not.toBeNull();

    storage.remove('test-key');
    expect(storage.get('test-key')).toBeNull();
  });

  it('should clear all items', () => {
    storage.set('key1', { data: 'value1' });
    storage.set('key2', { data: 'value2' });

    storage.clear();

    expect(storage.get('key1')).toBeNull();
    expect(storage.get('key2')).toBeNull();
  });

  it('should calculate approximate storage size', () => {
    storage.clear();

    storage.set('test', { data: 'x'.repeat(100) });
    const size = storage.getSize();

    // Should be > 0 after adding data
    expect(size).toBeGreaterThan(0);
  });
});

describe('Crypto utilities', () => {
  it('should generate random strings of correct length', () => {
    const str1 = generateRandomString(32);
    const str2 = generateRandomString(32);

    expect(str1).toHaveLength(64); // 32 bytes = 64 hex chars
    expect(str2).toHaveLength(64);
    expect(str1).not.toBe(str2); // Should be random
  });

  it('should hash strings consistently', async () => {
    const input = 'test-string';
    const hash1 = await hash(input);
    const hash2 = await hash(input);

    expect(hash1).toBe(hash2); // Same input = same hash
    expect(hash1).toHaveLength(64); // SHA-256 = 64 hex chars
  });

  it('should produce different hashes for different inputs', async () => {
    const hash1 = await hash('input1');
    const hash2 = await hash('input2');

    expect(hash1).not.toBe(hash2);
  });
});

describe('Storage keys', () => {
  it('should have all required storage keys defined', () => {
    expect(STORAGE_KEYS.AUTH_STATE).toBe('caro:auth:state');
    expect(STORAGE_KEYS.USER_PROFILE).toBe('caro:user:profile');
    expect(STORAGE_KEYS.PRIVACY_SETTINGS).toBe('caro:privacy:settings');
    expect(STORAGE_KEYS.LOCAL_COMMANDS).toBe('caro:local:commands');
  });
});
