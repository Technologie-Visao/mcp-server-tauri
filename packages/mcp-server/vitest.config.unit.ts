import { randomBytes } from 'crypto';
import { defineConfig } from 'vitest/config';

const TOKEN_BYTES = 32;

// PluginClient refuses to connect without a token; give unit tests a throwaway one.
// eslint-disable-next-line no-process-env
process.env.MCP_BRIDGE_TOKEN ||= randomBytes(TOKEN_BYTES).toString('base64url');

export default defineConfig({
   test: {
      globals: true,
      environment: 'node',
      testTimeout: 10000,
      hookTimeout: 5000,
      include: [ 'tests/unit/**/*.test.ts' ],
      // No global setup - unit tests don't need the Tauri app
   },
});
