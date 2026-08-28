import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('landing page is semantic, interactive, and error-free', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
  await page.goto('/');

  await expect(page).toHaveTitle(/Kube Permission Evidence/);
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  await expect(page.locator('main')).toHaveCount(1);
  await expect(page.locator('h1')).toHaveCount(1);
  await expect(page.getByRole('heading', { level: 1 })).toContainText('Trace every permission');
  await expect(page.locator('img:not([alt])')).toHaveCount(0);

  await page.getByLabel('Permission question').selectOption('deploy');
  await expect(page.getByText('DENIED', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Clear specimen' }).click();
  await expect(page.getByRole('heading', { name: 'No specimen selected' })).toBeVisible();

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact ?? ''))).toEqual([]);
  expect(errors).toEqual([]);
});

test('mobile layout keeps primary paths usable', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await expect(page.getByRole('link', { name: 'Install the CLI' })).toBeVisible();
  await page.getByRole('link', { name: 'Install the CLI' }).focus();
  await expect(page.getByRole('link', { name: 'Install the CLI' })).toBeFocused();
  await page.getByLabel('Permission question').selectOption('aggregate');
  await expect(page.getByText('ALLOWED · UNCERTAIN', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
});

test('license return is stored, stripped, and verified', async ({ page }) => {
  await page.route('https://api.sociobot.in/api/v1/products/kube-permission-evidence/verify?license=test-token', (route) => {
    void route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ valid: true, reason: 'ok', expires_at: null }) });
  });
  await page.goto('/?license=test-token');
  await expect(page).toHaveURL('/');
  await expect(page.getByText('Field kit license verified on this device.')).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('sb_license:kube-permission-evidence'))).toBe('test-token');
});

for (const route of ['/privacy/', '/terms/']) {
  test(`${route} has one main heading and no serious accessibility violations`, async ({ page }) => {
    await page.goto(route);
    await expect(page.locator('main')).toHaveCount(1);
    await expect(page.locator('h1')).toHaveCount(1);
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact ?? ''))).toEqual([]);
  });
}
