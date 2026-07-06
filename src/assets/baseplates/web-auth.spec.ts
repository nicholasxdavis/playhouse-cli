import { test, expect } from '@playwright/test';

// PLAYHOUSE: customize — auth flow placeholders for {{project_name}}
test.describe('auth', () => {
  test('login page loads', async ({ page }) => {
    await page.goto('{{url}}/login');
    await expect(page.locator('body')).toBeVisible();
  });

  // PLAYHOUSE: customize — fill credentials and assert redirect
  test.skip('user can sign in', async ({ page }) => {
    await page.goto('{{url}}/login');
  });
});
