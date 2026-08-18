import { test, expect } from '@playwright/test';

import { getSelectedTreeLabels } from './shared/test-apis';
import {
  clickSidebarSpec,
  fetchFirstSidebarLeaf,
  fetchSidebarLeafPair,
  getCurrentSpecId,
  sidebarRow,
} from './shared/spec-sidebar';
import { SPEC_VIEWER, gotoAndWaitForViewer } from './shared/viewers';

test.describe('spec-viewer — navigation and selection', () => {
  test('opening the sidebar drawer still allows selecting a spec', async ({ page }) => {
    test.setTimeout(90_000);

    await page.setViewportSize({ width: 700, height: 900 });
    await gotoAndWaitForViewer(page, SPEC_VIEWER);

    const spec = await fetchFirstSidebarLeaf(page);
    const toggle = page.getByRole('button', { name: 'Open specifications sidebar' });
    await expect(toggle).toBeVisible({ timeout: 20_000 });
    await toggle.click();

    const overlay = page.locator('.sidebar-overlay.visible');
    await expect(overlay).toBeVisible({ timeout: 10_000 });
    await expect
      .poll(() => overlay.evaluate((element) => window.getComputedStyle(element).backgroundColor), {
        timeout: 10_000,
        message: 'mobile drawer backdrop should stay transparent while preserving the outside-click target',
      })
      .toBe('rgba(0, 0, 0, 0)');

    const overlayBox = await overlay.boundingBox();
    expect(overlayBox, 'expected the drawer overlay to expose an outside-click hit area').not.toBeNull();
    if (!overlayBox) {
      throw new Error('expected visible drawer overlay bounds');
    }

    await page.mouse.click(overlayBox.x + Math.min(20, overlayBox.width / 2), Math.max(120, overlayBox.y + 120));

    await expect(page.getByRole('button', { name: 'Open specifications sidebar' })).toBeVisible({ timeout: 10_000 });
    await toggle.click();

    await clickSidebarSpec(page, spec);

    await expect
      .poll(() => getCurrentSpecId(page), {
        timeout: 10_000,
        message: 'clicking a tree leaf inside the open drawer should still navigate to the spec detail route',
      })
      .toBe(spec.id);

    await expect(page.getByRole('button', { name: 'Open specifications sidebar' })).toBeVisible({ timeout: 10_000 });
  });

  test('clicking a spec in the tree opens detail content in the main view', async ({ page }) => {
    test.setTimeout(90_000);

    await gotoAndWaitForViewer(page, SPEC_VIEWER);

    const subtitle = page.getByText('Browse by component or use the sidebar tree to open a specification.');
    await expect(subtitle).toBeVisible({ timeout: 20_000 });

    const spec = await fetchFirstSidebarLeaf(page);
    await clickSidebarSpec(page, spec);

    await expect
      .poll(() => getCurrentSpecId(page), {
        timeout: 10_000,
        message: 'clicking a tree leaf should push a canonical /specs/:id detail route',
      })
      .toBe(spec.id);

    await expect(subtitle).not.toBeVisible({ timeout: 10_000 });
    await expect(page.getByRole('button', { name: 'Body' })).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.spec-detail__title')).toBeVisible({ timeout: 10_000 });
    await expect
      .poll(() => getSelectedTreeLabels(page), {
        timeout: 10_000,
        message: 'selected file-tree row should track the active detail route',
      })
      .toEqual([spec.label]);
  });

  test('tree selection follows the active detail route when navigating browser history', async ({ page }) => {
    test.setTimeout(90_000);

    await gotoAndWaitForViewer(page, SPEC_VIEWER);

    const { first, second } = await fetchSidebarLeafPair(page);

    await clickSidebarSpec(page, first);
    await expect.poll(() => getCurrentSpecId(page), { timeout: 10_000 }).toBe(first.id);

    await clickSidebarSpec(page, second);
    await expect.poll(() => getCurrentSpecId(page), { timeout: 10_000 }).toBe(second.id);
    expect(second.id).not.toBe(first.id);

    await page.goBack();

    await expect
      .poll(() => getCurrentSpecId(page), {
        timeout: 10_000,
        message: 'detail route should return to the first selected spec after browser back',
      })
      .toBe(first.id);

    await expect
      .poll(() => getSelectedTreeLabels(page), {
        timeout: 10_000,
        message: 'selected file-tree row should track the restored detail route',
      })
      .toEqual([first.label]);
  });

  test('theme presets recolor selected spec rows (no fixed dark-blue background)', async ({ page }) => {
    test.setTimeout(90_000);

    await gotoAndWaitForViewer(page, SPEC_VIEWER);

    const spec = await fetchFirstSidebarLeaf(page);
    await clickSidebarSpec(page, spec);

    const row = sidebarRow(page, spec);
    await expect(row).toHaveClass(/selected/);

    const themeBtn = page.getByRole('button', { name: 'Theme settings' });
    await expect(themeBtn).toBeVisible({ timeout: 20_000 });
    await themeBtn.click();

    const panel = page.locator('.theme-settings');
    await expect(panel).toBeVisible({ timeout: 10_000 });

    const selectedRowBg = () => row.evaluate((element) => window.getComputedStyle(element).backgroundColor);

    await panel.getByRole('button', { name: 'Dark', exact: true }).click();
    await panel.getByRole('button', { name: 'Apply', exact: true }).click();

    const darkRowBg = await selectedRowBg();

    await panel.getByRole('button', { name: 'Paper', exact: true }).click();
    await panel.getByRole('button', { name: 'Apply', exact: true }).click();

    await expect
      .poll(
        () => page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--bg-primary').trim()),
        { timeout: 10_000, message: 'Paper preset should update primary background token' },
      )
      .toBe('#f5f0eb');

    const paperRowBg = await selectedRowBg();

    expect(darkRowBg, 'should be able to read selected row background in dark preset').toBeTruthy();
    expect(paperRowBg, 'should be able to read selected row background in paper preset').toBeTruthy();
    expect(paperRowBg, 'Paper preset should recolor selected row from dark preset value').not.toBe(darkRowBg);
    expect(
      paperRowBg,
      'selected row should not keep the legacy fixed dark-blue background in light themes',
    ).not.toBe('rgb(42, 58, 74)');
  });
});