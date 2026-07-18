// Tests for the evergreen model-selection and retry policy.
// Run: node --test .github/scripts/translate-multi-backend.test.js
//
// No network: every case drives the classifier or a stubbed thunk. The retry
// tests use base_delay_ms=1 so exponential backoff doesn't slow the suite.

const test = require('node:test');
const assert = require('node:assert');

const {
  classifyError,
  shouldFallback,
  extractStatus,
  withRetry,
  loadModelConfig,
  daysSince
} = require('./translate-multi-backend.js');

// Error shapes as each backend actually raises them.
const openaiError = status => Object.assign(new Error('Request failed'), { status });
const claudeError = (status, text = 'Not Found') =>
  new Error(`Claude API error: ${status} ${text}`);
const geminiError = (status, text = 'Not Found', body = '{"error":{"code":404}}') =>
  new Error(`Gemini API error: ${status} ${text} — ${body}`);

test('extractStatus reads the OpenAI SDK status property', () => {
  assert.strictEqual(extractStatus(openaiError(429)), 429);
});

test('extractStatus parses Claude and Gemini message formats', () => {
  assert.strictEqual(extractStatus(claudeError(503, 'Service Unavailable')), 503);
  assert.strictEqual(extractStatus(geminiError(404)), 404);
});

test('extractStatus is not confused by digits in the Gemini body slice', () => {
  // The body contains "404" but the real status is 400 — anchoring on the
  // "API error:" prefix is what keeps this correct.
  assert.strictEqual(extractStatus(geminiError(400, 'Bad Request')), 400);
});

test('extractStatus returns null when there is no status at all', () => {
  assert.strictEqual(extractStatus(new Error('socket hang up')), null);
});

test('429 and 5xx classify as retry, not fallback', () => {
  for (const status of [429, 500, 502, 503, 529]) {
    assert.strictEqual(classifyError(openaiError(status)), 'retry', `status ${status}`);
    assert.strictEqual(shouldFallback(openaiError(status)), false, `status ${status}`);
  }
});

test('404 classifies as gone and falls back immediately', () => {
  assert.strictEqual(classifyError(geminiError(404)), 'gone');
  assert.strictEqual(shouldFallback(geminiError(404)), true);
});

test('400 falls back only when the message names a missing model', () => {
  const retired = claudeError(400, 'model gpt-4-turbo-preview does not exist');
  assert.strictEqual(classifyError(retired), 'gone');

  // A plain 400 (e.g. a bad API key) is 'other' — still a fallback, but it is
  // not claimed to be a retired model.
  assert.strictEqual(classifyError(claudeError(400, 'Bad Request')), 'other');
  assert.strictEqual(shouldFallback(claudeError(400, 'Bad Request')), true);
});

test('unknown errors fall back rather than hanging the pipeline', () => {
  const parseFail = new SyntaxError('Unexpected token < in JSON at position 0');
  assert.strictEqual(classifyError(parseFail), 'other');
  assert.strictEqual(shouldFallback(parseFail), true);
});

test('withRetry retries a 429 and succeeds on a later attempt', async () => {
  let calls = 0;
  const result = await withRetry(
    async () => {
      calls++;
      if (calls < 3) throw openaiError(429);
      return 'ok';
    },
    { label: 'test', maxRetries: 3, baseDelayMs: 1 }
  );

  assert.strictEqual(result, 'ok');
  assert.strictEqual(calls, 3);
});

test('withRetry stops after max retries and rethrows', async () => {
  let calls = 0;
  await assert.rejects(
    withRetry(
      async () => { calls++; throw openaiError(503); },
      { label: 'test', maxRetries: 3, baseDelayMs: 1 }
    ),
    /503|Request failed/
  );

  // 1 initial attempt + 3 retries.
  assert.strictEqual(calls, 4);
});

test('withRetry does NOT retry a 404 — no backoff burned on a dead model', async () => {
  let calls = 0;
  await assert.rejects(
    withRetry(
      async () => { calls++; throw geminiError(404); },
      { label: 'test', maxRetries: 3, baseDelayMs: 1 }
    ),
    /404/
  );

  assert.strictEqual(calls, 1);
});

test('shipped config is valid and every chain entry is fully specified', () => {
  const config = loadModelConfig();

  assert.ok(config.chain.length >= 2, 'chain needs a fallback');
  assert.strictEqual(config.chain[0], 'gemini-2.5-flash');

  const keyEnvs = new Set();
  for (const id of config.chain) {
    const entry = config.models[id];
    assert.ok(entry.backend && entry.api_model && entry.api_key_env, `${id} underspecified`);
    assert.ok(!Number.isNaN(Date.parse(entry.last_verified)), `${id} last_verified unparseable`);
    keyEnvs.add(entry.api_key_env);
  }

  // A chain whose every model needs the same key is not a fallback chain.
  assert.ok(keyEnvs.size > 1, 'chain must span more than one provider credential');
});

test('daysSince treats a missing/garbage date as maximally stale', () => {
  assert.strictEqual(daysSince('not-a-date'), Infinity);
  assert.ok(daysSince('2020-01-01') > 1000);
});
