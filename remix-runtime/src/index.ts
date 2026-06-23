import { serve } from '@hono/node-server';
import app from './server';

const PORT = parseInt(process.env.PORT || '3000');
console.log(`Remix runtime listening on port ${PORT}`);

serve({
  fetch: app.fetch,
  port: PORT,
});
