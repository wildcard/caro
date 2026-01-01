/**
 * Cryptographic utilities for encrypting sensitive data
 * Used for OAuth tokens in localStorage
 */

/**
 * Generate a random encryption key from user's device
 * (In production, consider using Web Crypto API key derivation)
 */
export async function generateKey(): Promise<CryptoKey> {
  if (typeof window === 'undefined' || !window.crypto?.subtle) {
    throw new Error('Web Crypto API not available');
  }

  return await window.crypto.subtle.generateKey(
    {
      name: 'AES-GCM',
      length: 256,
    },
    true, // extractable
    ['encrypt', 'decrypt']
  );
}

/**
 * Derive encryption key from a passphrase
 * Useful for deterministic key generation
 */
export async function deriveKey(
  passphrase: string,
  salt: BufferSource
): Promise<CryptoKey> {
  if (typeof window === 'undefined' || !window.crypto?.subtle) {
    throw new Error('Web Crypto API not available');
  }

  const encoder = new TextEncoder();
  const passphraseKey = await window.crypto.subtle.importKey(
    'raw',
    encoder.encode(passphrase),
    'PBKDF2',
    false,
    ['deriveBits', 'deriveKey']
  );

  return await window.crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt,
      iterations: 100000,
      hash: 'SHA-256',
    },
    passphraseKey,
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  );
}

/**
 * Encrypt a string using AES-GCM
 */
export async function encrypt(
  plaintext: string,
  key: CryptoKey
): Promise<string> {
  if (typeof window === 'undefined' || !window.crypto?.subtle) {
    throw new Error('Web Crypto API not available');
  }

  const encoder = new TextEncoder();
  const data = encoder.encode(plaintext);

  // Generate random IV (initialization vector)
  const iv = window.crypto.getRandomValues(new Uint8Array(12));

  const encrypted = await window.crypto.subtle.encrypt(
    {
      name: 'AES-GCM',
      iv,
    },
    key,
    data
  );

  // Combine IV + encrypted data
  const combined = new Uint8Array(iv.length + encrypted.byteLength);
  combined.set(iv);
  combined.set(new Uint8Array(encrypted), iv.length);

  // Return as base64
  return btoa(String.fromCharCode(...combined));
}

/**
 * Decrypt a string using AES-GCM
 */
export async function decrypt(
  ciphertext: string,
  key: CryptoKey
): Promise<string> {
  if (typeof window === 'undefined' || !window.crypto?.subtle) {
    throw new Error('Web Crypto API not available');
  }

  // Decode from base64
  const combined = Uint8Array.from(atob(ciphertext), (c) => c.charCodeAt(0));

  // Extract IV and encrypted data
  const iv = combined.slice(0, 12);
  const data = combined.slice(12);

  const decrypted = await window.crypto.subtle.decrypt(
    {
      name: 'AES-GCM',
      iv,
    },
    key,
    data
  );

  const decoder = new TextDecoder();
  return decoder.decode(decrypted);
}

/**
 * Hash a string using SHA-256
 * Useful for generating deterministic IDs
 */
export async function hash(input: string): Promise<string> {
  if (typeof window === 'undefined' || !window.crypto?.subtle) {
    throw new Error('Web Crypto API not available');
  }

  const encoder = new TextEncoder();
  const data = encoder.encode(input);
  const hashBuffer = await window.crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
}

/**
 * Generate a random string (for PKCE, state, etc.)
 */
export function generateRandomString(length: number = 32): string {
  if (typeof window === 'undefined' || !window.crypto) {
    // Fallback for SSR
    return Math.random().toString(36).substring(2, 15);
  }

  const array = new Uint8Array(length);
  window.crypto.getRandomValues(array);
  return Array.from(array, (byte) => byte.toString(16).padStart(2, '0')).join(
    ''
  );
}

/**
 * Generate PKCE code verifier and challenge
 * Used for OAuth 2.0 PKCE flow
 */
export async function generatePKCE(): Promise<{
  codeVerifier: string;
  codeChallenge: string;
}> {
  const codeVerifier = generateRandomString(64);

  if (typeof window === 'undefined' || !window.crypto?.subtle) {
    // Fallback for SSR (not secure, only for development)
    return {
      codeVerifier,
      codeChallenge: codeVerifier,
    };
  }

  const hashed = await hash(codeVerifier);
  // Convert hex to base64url
  const bytes = new Uint8Array(
    hashed.match(/.{1,2}/g)!.map((byte) => parseInt(byte, 16))
  );
  const base64 = btoa(String.fromCharCode(...bytes));
  const codeChallenge = base64
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');

  return {
    codeVerifier,
    codeChallenge,
  };
}

/**
 * Secure storage wrapper for encrypted data
 */
export const secureStorage = {
  /**
   * Store encrypted value
   */
  setItem: async (
    key: string,
    value: string,
    encryptionKey: CryptoKey
  ): Promise<boolean> => {
    if (typeof window === 'undefined') return false;

    try {
      const encrypted = await encrypt(value, encryptionKey);
      localStorage.setItem(key, encrypted);
      return true;
    } catch (error) {
      console.error('Error storing encrypted value:', error);
      return false;
    }
  },

  /**
   * Retrieve decrypted value
   */
  getItem: async (
    key: string,
    encryptionKey: CryptoKey
  ): Promise<string | null> => {
    if (typeof window === 'undefined') return null;

    try {
      const encrypted = localStorage.getItem(key);
      if (!encrypted) return null;
      return await decrypt(encrypted, encryptionKey);
    } catch (error) {
      console.error('Error retrieving encrypted value:', error);
      return null;
    }
  },

  /**
   * Remove encrypted value
   */
  removeItem: (key: string): boolean => {
    if (typeof window === 'undefined') return false;

    try {
      localStorage.removeItem(key);
      return true;
    } catch (error) {
      console.error('Error removing encrypted value:', error);
      return false;
    }
  },
};
