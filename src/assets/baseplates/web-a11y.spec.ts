import { test, expect } from '@playwright/test';

// PLAYHOUSE: customize — hook up axe or similar a11y tooling for {{project_name}}
test('homepage has main landmark', async ({ page }) => {
  await page.goto('{{url}}');
  await expect(page.locator('main, [role="main"], body')).toBeVisible();
});
