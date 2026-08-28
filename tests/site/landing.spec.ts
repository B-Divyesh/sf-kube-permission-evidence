import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { readFileSync } from 'node:fs';

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

test('keyboard controls operate the full evidence-demo path', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', { name: 'Skip to main content' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/#main$/);

  const picker = page.getByLabel('Permission question');
  await picker.focus();
  await page.keyboard.press('End');
  await expect(page.getByText('ALLOWED · UNCERTAIN', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Clear specimen' }).focus();
  await page.keyboard.press('Space');
  await expect(page.getByRole('heading', { name: 'No specimen selected' })).toBeVisible();
});

for (const route of ['/', '/privacy/', '/terms/']) {
  test(`${route} has 44px mobile touch targets`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(route);
    const undersized = await page.locator('a[href], button, input, select, summary').evaluateAll((elements) =>
      elements
        .filter((element) => {
          const style = getComputedStyle(element);
          const rect = element.getBoundingClientRect();
          return style.visibility !== 'hidden' && style.display !== 'none' && rect.width > 0 && rect.height > 0;
        })
        .map((element) => {
          const rect = element.getBoundingClientRect();
          return { label: element.getAttribute('aria-label') ?? element.textContent?.trim() ?? element.tagName, width: rect.width, height: rect.height };
        })
        .filter(({ width, height }) => width < 44 || height < 44),
    );
    expect(undersized).toEqual([]);
  });
}

test('license return is stored, stripped, and verified', async ({ page }) => {
  await page.route('https://api.sociobot.in/api/v1/products/kube-permission-evidence/verify?license=test-token', (route) => {
    void route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ valid: true, reason: 'ok', expires_at: null }) });
  });
  await page.goto('/?license=test-token');
  await expect(page).toHaveURL('/');
  await expect(page.getByText('Field kit license verified on this device.')).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('sb_license:kube-permission-evidence'))).toBe('test-token');
});

test('versioned service worker claims immediately and supports an offline reload', async ({ page, context }) => {
  await page.goto('/');
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.waitForFunction(() => navigator.serviceWorker.controller !== null);
  await page.reload();
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByText('You’re offline.')).toBeVisible();
  await page.getByLabel('Permission question').selectOption('deploy');
  await expect(page.getByText('DENIED', { exact: true })).toBeVisible();
  await context.setOffline(false);
});

test('deployment artifact contains native security, cache, and worker update policy', async () => {
  const policy = JSON.parse(readFileSync('dist/site/staticwebapp.config.json', 'utf8')) as {
    globalHeaders: Record<string, string>;
    routes: Array<{ route: string; headers: Record<string, string> }>;
  };
  expect(policy.globalHeaders['Content-Security-Policy']).toContain("default-src 'self'");
  expect(policy.globalHeaders['Content-Security-Policy']).toContain('https://api.sociobot.in');
  expect(policy.globalHeaders['Permissions-Policy']).toContain('camera=()');
  expect(policy.routes.find(({ route }) => route === '/assets/*')?.headers['Cache-Control']).toBe('public, max-age=31536000, immutable');
  expect(policy.routes.find(({ route }) => route === '/sw.js')?.headers['Cache-Control']).toContain('no-store');

  const worker = readFileSync('dist/site/sw.js', 'utf8');
  expect(worker).toContain("kpe-field-guide-v2");
  expect(worker).toContain('self.skipWaiting()');
  expect(worker).toContain('self.clients.claim()');
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
