import { test, expect } from '@playwright/test';

// PLAYHOUSE: customize — smoke test for {{project_name}}
test('homepage loads', async ({ page }) => {
  await page.goto('{{url}}');
  await expect(page).toHaveTitle(/.+/);
});
