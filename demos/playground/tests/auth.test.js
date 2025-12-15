const { hashPassword, verifyPassword } = require('../src/auth');

describe('Authentication', () => {
  test('should hash password', async () => {
    const hash = await hashPassword('secret123');
    expect(hash).toBeDefined();
  });

  test('should verify password', async () => {
    const hash = await hashPassword('secret123');
    const isValid = await verifyPassword('secret123', hash);
    expect(isValid).toBe(true);
  });
});
