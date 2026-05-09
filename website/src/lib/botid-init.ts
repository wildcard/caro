import { initBotId } from 'botid/client/core';

initBotId({
  protect: [{ path: '/api/waitlist', method: 'POST' }],
});
