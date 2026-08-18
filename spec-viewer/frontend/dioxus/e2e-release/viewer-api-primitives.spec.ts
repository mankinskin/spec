import { expect, test } from '@playwright/test';

import {
  buildSpecDetailUrl,
  clickSidebarSpec,
  fetchFirstSidebarLeaf,
  fetchSidebarLeafPair,
  getCurrentSpecId,
} from './shared/spec-sidebar';
import { SPEC_VIEWER, gotoAndWaitForViewer } from './shared/viewers';

test.describe('spec-viewer — shared detail-shell primitives', () => {
  test('P5.5 HeaderActions: shared header buttons render', async ({ page }) => {
    test.setTimeout(60_000);
    await gotoAndWaitForViewer(page, SPEC_VIEWER);
    await expect(page.getByRole('button', { name: 'Home' })).toBeVisible({ timeout: 10_000 });
    await expect(page.getByRole('button', { name: 'Toggle filters' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Theme settings' })).toBeVisible();
  });

  test('P5.4 Overlay: theme settings open in a role=dialog modal-backdrop', async ({ page }, testInfo) => {
    test.setTimeout(60_000);
    await gotoAndWaitForViewer(page, SPEC_VIEWER);

    await page.getByRole('button', { name: 'Theme settings', exact: true }).click();

    const dialog = page.locator('.modal-backdrop[role="dialog"][aria-label="Theme settings"]');
    await expect(dialog).toBeVisible({ timeout: 10_000 });

    const panel = page.locator('.theme-settings.glass-panel');
    await expect(panel).toBeVisible();

    const backdropStyles = await dialog.evaluate((element) => {
      const styles = getComputedStyle(element as HTMLElement);
      return {
        backgroundColor: styles.backgroundColor,
        position: styles.position,
        zIndex: styles.zIndex,
      };
    });
    expect(backdropStyles.position).toBe('fixed');
    expect(backdropStyles.zIndex).not.toBe('auto');
    expect(backdropStyles.backgroundColor).not.toBe('rgba(0, 0, 0, 0)');

    await testInfo.attach('spec-viewer-theme-settings-panel', {
      body: await panel.screenshot(),
      contentType: 'image/png',
    });

    await panel.locator('button', { hasText: '✕' }).first().click();
    await expect(panel).not.toBeVisible({ timeout: 5_000 });
  });

  test('sidebar selection opens a canonical detail route and breadcrumb trail', async ({ page }) => {
    test.setTimeout(90_000);
    await gotoAndWaitForViewer(page, SPEC_VIEWER);

    const spec = await fetchFirstSidebarLeaf(page);
    await clickSidebarSpec(page, spec);

    const breadcrumbs = page.getByRole('navigation', { name: 'Breadcrumb' });
    await expect(breadcrumbs).toBeVisible();
    await expect
      .poll(() => getCurrentSpecId(page), { timeout: 10_000 })
      .toBe(spec.id);
    await expect(breadcrumbs).toContainText('Specs');
    await expect(breadcrumbs).toContainText('Graph');
    await expect(breadcrumbs).toContainText(spec.id);
    await expect(page.locator('.spec-detail__title')).toBeVisible();
    await expect(page.locator('.spec-detail__tab--active')).toHaveText('Body');
  });

  test('deep-linked detail routes restore the selected tab from the query string', async ({ page }) => {
    test.setTimeout(90_000);
    await gotoAndWaitForViewer(page, SPEC_VIEWER);
    const spec = await fetchFirstSidebarLeaf(page);

    await page.goto(`${SPEC_VIEWER.url}${buildSpecDetailUrl(spec.id, 'sections')}`, { waitUntil: 'domcontentloaded' });
    await page.locator(SPEC_VIEWER.readySelector).first().waitFor({
      state: 'visible',
      timeout: SPEC_VIEWER.readyTimeout,
    });

    await expect
      .poll(() => getCurrentSpecId(page), { timeout: 10_000 })
      .toBe(spec.id);

    await expect(page.locator('.spec-detail__tab--active')).toHaveText('Sections');
    await expect(page.getByRole('navigation', { name: 'Breadcrumb' })).toContainText(spec.id);
  });

  test('shared Home action returns to the browse route from detail and graph headers', async ({ page }) => {
    test.setTimeout(90_000);
    await gotoAndWaitForViewer(page, SPEC_VIEWER);
    const spec = await fetchFirstSidebarLeaf(page);

    await page.goto(`${SPEC_VIEWER.url}${buildSpecDetailUrl(spec.id)}`, {
      waitUntil: 'domcontentloaded',
    });
    await page.locator(SPEC_VIEWER.readySelector).first().waitFor({
      state: 'visible',
      timeout: SPEC_VIEWER.readyTimeout,
    });

    const homeButton = page.getByRole('button', { name: 'Home' });
    await expect(homeButton).toBeVisible({ timeout: 10_000 });
    await homeButton.click();

    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'detail-page Home should navigate back to the canonical browse route',
      })
      .toBe(`${SPEC_VIEWER.url}/specs`);

    await page.goto(`${SPEC_VIEWER.url}/specs/graph`, {
      waitUntil: 'domcontentloaded',
    });
    await page.locator(SPEC_VIEWER.readySelector).first().waitFor({
      state: 'visible',
      timeout: SPEC_VIEWER.readyTimeout,
    });

    await expect(homeButton).toBeVisible({ timeout: 10_000 });
    await homeButton.click();

    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'graph-page Home should navigate back to the canonical browse route',
      })
      .toBe(`${SPEC_VIEWER.url}/specs`);
  });

  test('selecting a second sidebar spec preserves the current detail view', async ({ page }) => {
    test.setTimeout(90_000);
    await gotoAndWaitForViewer(page, SPEC_VIEWER);

    const { first, second } = await fetchSidebarLeafPair(page);

    await clickSidebarSpec(page, first);
    await expect.poll(() => getCurrentSpecId(page), { timeout: 10_000 }).toBe(first.id);

    await page.getByRole('button', { name: 'Sections' }).click();
    await expect(page.locator('.spec-detail__tab--active')).toHaveText('Sections');

    await clickSidebarSpec(page, second);
    await expect.poll(() => getCurrentSpecId(page), { timeout: 10_000 }).toBe(second.id);
    await expect(page.locator('.spec-detail__tab--active')).toHaveText('Sections');
  });
});
