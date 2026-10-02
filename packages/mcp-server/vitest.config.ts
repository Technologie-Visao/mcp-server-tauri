import { randomBytes } from 'crypto';
import { defineConfig } from 'vitest/config';

const TOKEN_BYTES = 32;

// The bridge refuses to start without a token. Generate a fresh one per run and
// export it so both the globally spawned test-app and the test workers inherit it.
// eslint-disable-next-line no-process-env
process.env.MCP_BRIDGE_TOKEN ||= randomBytes(TOKEN_BYTES).toString('base64url');

export default defineConfig({
   test: {
      globals: true,
      environment: 'node',
      testTimeout: 10000, // 10s for individual tests
      hookTimeout: 5000, // 5s for hooks
      include: [ 'tests/**/*.test.ts' ],
      maxConcurrency: 1, // Run tests sequentially to avoid port conflicts
      fileParallelism: false, // Disable file-level parallelism
      pool: 'forks', // Use separate processes for isolation
      globalSetup: './vitest.global-setup.ts', // Start app once globally
   },
});
