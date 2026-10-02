import { describe, it, expect } from 'vitest';
import WebSocket from 'ws';
import { getTestAppPort } from '../test-utils';

type HandshakeOutcome = 'open' | number;

/**
 * Opens a raw WebSocket to the bridge and reports either 'open' or the HTTP
 * status the handshake was rejected with.
 */
function attemptHandshake(headers: Record<string, string>): Promise<HandshakeOutcome> {
   return new Promise((resolve, reject) => {
      const ws = new WebSocket(`ws://127.0.0.1:${getTestAppPort()}`, { headers });

      ws.on('open', () => {
         ws.close();
         resolve('open');
      });
      ws.on('unexpected-response', (_req, res) => {
         resolve(res.statusCode ?? 0);
         ws.terminate();
      });
      ws.on('error', reject);
   });
}

/**
 * E2E tests for bridge handshake hardening: bearer token and Origin rejection.
 */
describe('MCP Bridge authentication', () => {
   const token = process.env.MCP_BRIDGE_TOKEN ?? '';

   it('accepts the correct bearer token', async () => {
      expect(await attemptHandshake({ Authorization: `Bearer ${token}` })).toBe('open');
   });

   it('rejects a missing token', async () => {
      expect(await attemptHandshake({})).toBe(401);
   });

   it('rejects a wrong token', async () => {
      expect(await attemptHandshake({ Authorization: `Bearer ${token}x` })).toBe(401);
   });

   it('rejects any Origin header even with the correct token', async () => {
      for (const origin of [ 'http://evil.example', 'null' ]) {
         expect(await attemptHandshake({ Authorization: `Bearer ${token}`, Origin: origin })).toBe(403);
      }
   });
});
