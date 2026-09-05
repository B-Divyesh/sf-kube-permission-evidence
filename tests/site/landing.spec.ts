import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { readFileSync } from 'node:fs';

test('landing page states the job, audience, and sample action first', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  await page.goto('/');

  await expect(page).toHaveTitle('Kube Permission Evidence — prove Kubernetes access');
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  await expect(page.locator('main')).toHaveCount(1);
  await expect(page.locator('h1')).toHaveCount(1);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Trace Kubernetes access to every granting rule');
  await expect(page.getByText(/For Kubernetes operators preparing audits/)).toBeVisible();
  await expect(page.getByRole('link', { name: 'Try it with sample data' })).toBeVisible();
  await expect(page.locator('img:not([alt])')).toHaveCount(0);

  await page.getByLabel('Permission question').selectOption('exec');
  await expect(page.getByText('DENIED', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Clear result' }).click();
  await expect(page.getByRole('heading', { name: 'No result selected' })).toBeVisible();

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact ?? ''))).toEqual([]);
  expect(errors).toEqual([]);
});

test('first-screen copy keeps accessible contrast during default-motion entrance', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.goto('/');
  const opacity = await page.locator('.hero-copy').evaluate((hero) => {
    const animation = hero.getAnimations().at(0);
    if (!animation) throw new Error('expected the default hero entrance animation');
    animation.pause();
    animation.currentTime = 200;
    return getComputedStyle(hero).opacity;
  });
  expect(opacity).toBe('1');

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact ?? ''))).toEqual([]);
});

test('one click opens a populated isolated demo with reset and exit', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(() => localStorage.setItem('sb_license:kube-permission-evidence', 'real-license'));
  await page.getByRole('link', { name: 'Try it with sample data' }).click();
  await expect(page).toHaveURL('/demo/');
  await expect(page).toHaveTitle('Demo — Kube Permission Evidence');
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await expect(page.getByText('ALLOWED', { exact: true })).toBeVisible();
  await expect(page.getByText('payments / alice-secrets')).toBeVisible();

  await page.getByLabel('Permission question').selectOption('health');
  await expect(page.getByText('DENIED', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => sessionStorage.getItem('demo:kpe:selected-case'))).toBe('health');
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.getByText('payments / alice-secrets')).toBeVisible();
  expect(await page.evaluate(() => sessionStorage.getItem('demo:kpe:selected-case'))).toBeNull();
  expect(await page.evaluate(() => localStorage.getItem('sb_license:kube-permission-evidence'))).toBe('real-license');

  await page.getByRole('link', { name: 'Start for real' }).click();
  await expect(page).toHaveURL('/');
  expect(await page.evaluate(() => localStorage.getItem('sb_license:kube-permission-evidence'))).toBe('real-license');
});

test('mobile layout keeps the sample action and result usable', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  const sample = page.getByRole('link', { name: 'Try it with sample data' });
  await expect(sample).toBeVisible();
  expect((await sample.boundingBox())?.y).toBeLessThan(844);
  await sample.click();
  await page.getByLabel('Permission question').selectOption('deploy');
  await expect(page.getByText('ALLOWED', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
});

test('keyboard controls operate the full demo path', async ({ page }) => {
  await page.goto('/demo/');
  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', { name: 'Skip to main content' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/#main$/);

  const picker = page.getByLabel('Permission question');
  await picker.focus();
  await page.keyboard.press('End');
  await expect(page.getByText('DENIED', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Clear result' }).focus();
  await page.keyboard.press('Space');
  await expect(page.getByRole('heading', { name: 'No result selected' })).toBeVisible();
  await page.getByRole('button', { name: 'Reset demo' }).focus();
  await page.keyboard.press('Enter');
  await expect(page.getByText('payments / alice-secrets')).toBeVisible();
});

for (const route of ['/', '/demo/', '/privacy/', '/terms/', '/404.html']) {
  test(`${route} has complete structure, metadata, and accessible mobile targets`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(route);
    await expect(page.locator('main')).toHaveCount(1);
    await expect(page.locator('h1')).toHaveCount(1);
    await expect(page.locator('link[rel="canonical"]')).toHaveCount(1);
    await expect(page.locator('meta[property="og:image"]')).toHaveCount(1);
    await expect(page.locator('meta[name="twitter:card"]')).toHaveAttribute('content', 'summary_large_image');
    await expect(page.locator('link[rel="apple-touch-icon"]')).toHaveCount(1);
    await expect(page.getByText('Built by Param Factory')).toBeVisible();
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
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact ?? ''))).toEqual([]);
  });
}

test('license return is stored, stripped, and verified', async ({ page }) => {
  await page.route('https://api.sociobot.in/api/v1/products/kube-permission-evidence/verify?license=test-token', (route) => {
    void route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ valid: true, reason: 'ok', expires_at: null }) });
  });
  await page.goto('/?license=test-token');
  await expect(page).toHaveURL('/');
  await expect(page.getByText('Field Kit license verified on this device.')).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('sb_license:kube-permission-evidence'))).toBe('test-token');
  expect(JSON.parse(await page.evaluate(() => localStorage.getItem('sb_license_verdict:kube-permission-evidence') ?? '{}'))).toMatchObject({
    token: 'test-token', valid: true, reason: 'ok',
  });
});

test('a cached verdict is never reused for a replacement license token', async ({ page }) => {
  const verified: string[] = [];
  await page.route('https://api.sociobot.in/api/v1/products/kube-permission-evidence/verify?license=*', async (route) => {
    const token = new URL(route.request().url()).searchParams.get('license') ?? '';
    verified.push(token);
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ valid: token === 'valid-one', reason: token === 'valid-one' ? 'ok' : 'invalid', expires_at: null }),
    });
  });

  await page.goto('/?license=valid-one');
  await expect(page.getByText('Field Kit license verified on this device.')).toBeVisible();
  await expect(page.locator('#unlocked-tools')).toBeVisible();

  await page.goto('/?license=invalid-two');
  await expect(page).toHaveURL('/');
  await expect(page.getByText(/License is not active/)).toBeVisible();
  await expect(page.locator('#unlocked-tools')).toBeHidden();
  expect(verified).toEqual(['valid-one', 'invalid-two']);
});

test('license return URLs never enter Cache Storage', async ({ page }) => {
  await page.route('https://api.sociobot.in/api/v1/products/kube-permission-evidence/verify?license=qa-cache-secret', (route) => {
    void route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ valid: false, reason: 'invalid', expires_at: null }) });
  });
  await page.goto('/');
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.waitForFunction(() => navigator.serviceWorker.controller !== null);

  await page.goto('/?license=qa-cache-secret');
  await expect(page).toHaveURL('/');
  await expect(page.getByText(/License is not active/)).toBeVisible();
  const cachedUrls = await page.evaluate(async () => {
    const urls: string[] = [];
    for (const cacheName of await caches.keys()) {
      urls.push(...(await (await caches.open(cacheName)).keys()).map((request) => request.url));
    }
    return urls;
  });
  expect(cachedUrls.filter((url) => new URL(url).searchParams.has('license'))).toEqual([]);
});

test('deployment policy defines a real 404 and durable cache rules', async () => {
  const policy = JSON.parse(readFileSync('dist/site/staticwebapp.config.json', 'utf8')) as {
    responseOverrides: Record<string, { rewrite: string; statusCode: number }>;
    globalHeaders: Record<string, string>;
    routes: Array<{ route: string; headers: Record<string, string> }>;
  };
  expect(policy.responseOverrides['404']).toEqual({ rewrite: '/404.html', statusCode: 404 });
  expect(policy.globalHeaders['Content-Security-Policy']).toContain("default-src 'self'");
  expect(policy.globalHeaders['Content-Security-Policy']).toContain('https://api.sociobot.in');
  expect(policy.routes.find(({ route }) => route === '/assets/*')?.headers['Cache-Control']).toBe('public, max-age=31536000, immutable');
  expect(policy.routes.find(({ route }) => route === '/sw.js')?.headers['Cache-Control']).toContain('no-store');
});
