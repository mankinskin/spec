import { expect, test } from '@playwright/test';

import { SPEC_VIEWER, gotoAndWaitForViewer } from './shared/viewers';

test.describe('spec-viewer — root browse navigation', () => {
  test('root browse cards navigate to canonical detail routes and tree redirects home', async ({ page }) => {
    test.setTimeout(90_000);

    await gotoAndWaitForViewer(page, SPEC_VIEWER);

    const firstCard = page.locator('.card.card--clickable').first();
    await expect(firstCard).toBeVisible({ timeout: 20_000 });
    await firstCard.click();

    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'browse-card clicks should navigate to a canonical /specs/:id detail route',
      })
      .toMatch(/\/specs\/[^?#]+$/);

    expect(
      page.url(),
      'the root browse route should no longer encode selected specs in the URL hash',
    ).not.toContain('#id=');

    await expect(page.getByRole('button', { name: 'Body' })).toBeVisible({ timeout: 10_000 });

    await page.goto(`${SPEC_VIEWER.url}/specs/tree`, { waitUntil: 'domcontentloaded' });
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'legacy /specs/tree links should normalize back to the root browse route',
      })
      .toBe(`${SPEC_VIEWER.url}/specs`);

    await expect(page.getByRole('heading', { name: 'Specifications' }).first()).toBeVisible({
      timeout: 10_000,
    });
  });
});