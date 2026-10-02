import { afterEach, describe, expect, it } from 'vitest';
import { WebSocketServer } from 'ws';
import type { AddressInfo } from 'net';
import type { IncomingMessage } from 'http';

import { PluginClient } from '../../src/driver/plugin-client.js';

describe('PluginClient authentication', () => {
   const originalToken = process.env.MCP_BRIDGE_TOKEN;

   afterEach(() => {
      process.env.MCP_BRIDGE_TOKEN = originalToken;
   });

   it('refuses to connect without MCP_BRIDGE_TOKEN', async () => {
      delete process.env.MCP_BRIDGE_TOKEN;
      const client = new PluginClient('127.0.0.1', 1);

      await expect(client.connect()).rejects.toThrow('MCP_BRIDGE_TOKEN is not set');
      client.disconnect();
   });

   it('sends the token as a bearer header and no Origin', async () => {
      const server = new WebSocketServer({ host: '127.0.0.1', port: 0 }),
            requests: IncomingMessage[] = [];

      server.on('connection', (_socket, req) => {
         requests.push(req);
      });
      await new Promise<void>((resolve) => { server.once('listening', resolve); });

      const { port } = server.address() as AddressInfo,
            client = new PluginClient('127.0.0.1', port);

      try {
         await client.connect();
         expect(requests[0]?.headers.authorization).toBe(`Bearer ${originalToken}`);
         expect(requests[0]?.headers.origin).toBeUndefined();
         expect(requests[0]?.url).not.toContain(originalToken);
      } finally {
         client.disconnect();
         await new Promise<void>((resolve) => {
            server.close(() => {
               resolve();
            });
         });
      }
   });
});
