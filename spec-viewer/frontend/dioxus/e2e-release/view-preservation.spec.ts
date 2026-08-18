import { expect, test, type Page } from '@playwright/test';

import { SPEC_VIEWER, gotoAndWaitForViewer } from './shared/viewers';

const GRAPH_FETCH_ERROR = 'Failed to load graph: fetch error: TypeError: Failed to fetch';
const SPECS_FETCH_ERROR = 'Failed to load specifications: fetch error: TypeError: Failed to fetch';

test.use({
  headless: false,
  launchOptions: {
    args: [
      '--enable-unsafe-webgpu',
      '--enable-features=Vulkan',
      '--use-vulkan=swiftshader',
      '--use-webgpu-adapter=swiftshader',
    ],
  },
});

async function fetchFirstSpecId(page: Page): Promise<string> {
  const specId = await page.evaluate(async () => {
    const response = await fetch('/api/specs?limit=1');
    if (!response.ok) {
      throw new Error(`Failed to load specs: ${response.status}`);
    }

    const payload = (await response.json()) as {
      items?: Array<{ id?: string }>;
    };

    return payload.items?.[0]?.id ?? null;
  });

  expect(specId, 'expected the spec-viewer API to return at least one spec').toBeTruthy();
  return specId as string;
}

async function expectSectionsView(page: Page): Promise<string> {
  await expect
    .poll(() => page.url(), {
      timeout: 15_000,
      message: 'expected the selected spec to open on the preserved sections detail view',
    })
    .toMatch(/\/specs\/[^?]+\?view=sections$/);
  await expect(page.getByRole('button', { name: 'Sections' })).toHaveClass(/active/, {
    timeout: 10_000,
  });
  return page.url();
}

async function waitForGraph(page: Page): Promise<void> {
  let lastError: unknown;

  for (let attempt = 0; attempt < 2; attempt += 1) {
    if (attempt > 0) {
      await page.reload({ waitUntil: 'domcontentloaded' });
    }

    await page.locator(SPEC_VIEWER.readySelector).first().waitFor({
      state: 'visible',
      timeout: SPEC_VIEWER.readyTimeout,
    });

    try {
      await expect(page.locator('#spec-graph3d-container')).toBeAttached({ timeout: 30_000 });
      await page.waitForFunction(() => {
        return Array.from(document.querySelectorAll('#spec-graph3d-container .graph-node-card')).some(
          (card) => (card as HTMLElement).style.display !== 'none',
        );
      }, null, { timeout: 30_000 });
      return;
    } catch (error) {
      lastError = error;
      if (attempt === 1 || !(await page.getByText(GRAPH_FETCH_ERROR).isVisible())) {
        throw error;
      }
    }
  }

  throw lastError instanceof Error ? lastError : new Error('expected spec graph to load');
}

async function waitForBrowseCards(page: Page): Promise<void> {
  let lastError: unknown;

  for (let attempt = 0; attempt < 2; attempt += 1) {
    if (attempt > 0) {
      await page.reload({ waitUntil: 'domcontentloaded' });
    }

    await page.locator(SPEC_VIEWER.readySelector).first().waitFor({
      state: 'visible',
      timeout: SPEC_VIEWER.readyTimeout,
    });

    try {
      await expect(page.locator('.card.card--clickable').nth(1)).toBeVisible({ timeout: 20_000 });
      return;
    } catch (error) {
      lastError = error;
      if (attempt === 1 || !(await page.getByText(SPECS_FETCH_ERROR).isVisible())) {
        throw error;
      }
    }
  }

  throw lastError instanceof Error ? lastError : new Error('expected browse cards to load');
}

async function nthVisibleGraphCardIndex(page: Page, visibleOffset: number): Promise<number> {
  return page.evaluate((offset) => {
    const cards = Array.from(document.querySelectorAll('#spec-graph3d-container .graph-node-card'));
    const visible = cards.filter((card) => (card as HTMLElement).style.display !== 'none');
    const target = visible[offset] as HTMLElement | undefined;
    if (!target) {
      return -1;
    }

    return cards.findIndex((card) => card === target);
  }, visibleOffset);
}

async function openDifferentGraphDetailFromPreview(
  page: Page,
  sourceUrl: string,
): Promise<string> {
  const sourcePath = new URL(sourceUrl).pathname;

  for (let visibleOffset = 0; visibleOffset < 6; visibleOffset += 1) {
    const cardIndex = await nthVisibleGraphCardIndex(page, visibleOffset);
    if (cardIndex < 0) {
      break;
    }

    const card = page.locator('#spec-graph3d-container .graph-node-card').nth(cardIndex);
    await card.click();

    const preview = page.locator('[data-testid="spec-preview"]');
    await expect(preview).toBeVisible({ timeout: 10_000 });
    await preview.getByRole('button', { name: /View details/i }).click();

    const targetUrl = await expectSectionsView(page);
    if (new URL(targetUrl).pathname !== sourcePath) {
      return targetUrl;
    }

    await page.getByRole('link', { name: '🌐 Graph' }).click();
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'returning to the graph from the same-spec detail should stay in the current SPA session',
      })
      .toBe(`${SPEC_VIEWER.url}/specs/graph`);
    await waitForGraph(page);
  }

  throw new Error('expected a graph preview selection to open a different specification');
}

test.describe('spec-viewer - detail view preservation', () => {
  test('root browse selection preserves the current detail view', async ({ page }) => {
    test.setTimeout(90_000);

    await gotoAndWaitForViewer(page, SPEC_VIEWER);

    const firstCard = page.locator('.card.card--clickable').first();
    const secondCard = page.locator('.card.card--clickable').nth(1);
    await waitForBrowseCards(page);

    await firstCard.click();
    await page.getByRole('button', { name: 'Sections' }).click();
    const sourceUrl = await expectSectionsView(page);

    await page.getByRole('link', { name: 'Specs', exact: true }).click();
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'detail navigation should return to the canonical root browse route',
      })
      .toBe(`${SPEC_VIEWER.url}/specs`);

    await waitForBrowseCards(page);
    await secondCard.click();

    const targetUrl = await expectSectionsView(page);
    expect(
      targetUrl,
      'clicking a different browse card should keep the sections view instead of resetting to body',
    ).not.toBe(sourceUrl);
  });

  test('browser history restores browse, detail view, and graph state', async ({ page }) => {
    test.setTimeout(120_000);

    await gotoAndWaitForViewer(page, SPEC_VIEWER);

    const firstCard = page.locator('.card.card--clickable').first();
    const secondCard = page.locator('.card.card--clickable').nth(1);
    await waitForBrowseCards(page);

    await firstCard.click();
    await page.getByRole('button', { name: 'Sections' }).click();
    await expectSectionsView(page);

    await page.getByRole('link', { name: 'Specs', exact: true }).click();
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'detail navigation should record the root browse route in browser history',
      })
      .toBe(`${SPEC_VIEWER.url}/specs`);

    await waitForBrowseCards(page);
    await secondCard.click();
    const secondSectionsUrl = await expectSectionsView(page);

    await page.getByRole('link', { name: '🌐 Graph' }).click();
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'detail navigation should record the graph route in browser history',
      })
      .toBe(`${SPEC_VIEWER.url}/specs/graph`);
    await waitForGraph(page);

    await page.goBack({ waitUntil: 'domcontentloaded' });
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'browser back should restore the previously selected spec and its sections view',
      })
      .toBe(secondSectionsUrl);
    await expect(page.getByRole('button', { name: 'Sections' })).toHaveClass(/active/, {
      timeout: 10_000,
    });

    await page.goBack({ waitUntil: 'domcontentloaded' });
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'browser back should restore the root browse state after the detail page',
      })
      .toBe(`${SPEC_VIEWER.url}/specs`);
    await expect(page.getByRole('heading', { name: 'Specifications' }).first()).toBeVisible({
      timeout: 10_000,
    });

    await page.goForward({ waitUntil: 'domcontentloaded' });
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'browser forward should restore the selected spec and preserved sections view',
      })
      .toBe(secondSectionsUrl);
    await expect(page.getByRole('button', { name: 'Sections' })).toHaveClass(/active/, {
      timeout: 10_000,
    });

    await page.goForward({ waitUntil: 'domcontentloaded' });
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'browser forward should restore the graph collection view',
      })
      .toBe(`${SPEC_VIEWER.url}/specs/graph`);
    await waitForGraph(page);
  });
});

test.describe('spec-viewer - graph detail view preservation', () => {
  test('graph preview details preserve the current detail view', async ({ page }) => {
    test.setTimeout(120_000);

    await gotoAndWaitForViewer(page, SPEC_VIEWER);
    const specId = await fetchFirstSpecId(page);

    await page.goto(`${SPEC_VIEWER.url}/specs/${specId}`, { waitUntil: 'domcontentloaded' });
    await page.getByRole('button', { name: 'Sections' }).click();
    const sourceUrl = await expectSectionsView(page);

    await page.getByRole('link', { name: '🌐 Graph' }).click();
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'detail navigation should reach the graph overview route',
      })
      .toBe(`${SPEC_VIEWER.url}/specs/graph`);
    await waitForGraph(page);

    const targetUrl = await openDifferentGraphDetailFromPreview(page, sourceUrl);
    expect(
      targetUrl,
      'graph preview detail navigation should keep the preserved sections view when switching specs',
    ).not.toBe(sourceUrl);
  });
});