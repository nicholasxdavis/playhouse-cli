import { test, expect } from '@playwright/test';

// PLAYHOUSE: customize — health endpoint for {{project_name}}
test('GET /health returns 200', async ({ request }) => {
  const response = await request.get('{{url}}/health');
  expect(response.status()).toBe(200);
});
