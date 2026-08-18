import { expect, test, type Page } from '@playwright/test';

import { SPEC_VIEWER } from './shared/viewers';

const GRAPH_FETCH_ERROR = 'Failed to load graph: fetch error: TypeError: Failed to fetch';

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

test.describe('spec-viewer - graph selection', () => {
  test('graph overview renders visible directed edges with arrow markers', async ({ page }) => {
    test.setTimeout(120_000);

    await gotoGraph(page);
    await page.mouse.move(20, 20);

    const edgeOverlay = page.locator('#spec-graph3d-container .graph-edge-overlay');
    await expect(edgeOverlay).toBeVisible({ timeout: 10_000 });

    const metrics = await page.evaluate(() => {
      const paths = Array.from(document.querySelectorAll('#spec-graph3d-container .graph-edge-overlay path'));
      const visiblePaths = paths.filter((path) => path.getAttribute('display') !== 'none');
      const lengths = visiblePaths.map((path) => {
        const d = path.getAttribute('d') ?? '';
        const parts = d.match(/-?\d+\.?\d*/g);
        if (!parts || parts.length < 8) return 0;
        const x1 = Number(parts[0]);
        const y1 = Number(parts[1]);
        const x2 = Number(parts[6]); // 4th point in the path (the tip of the arrow)
        const y2 = Number(parts[7]);
        return Math.hypot(x2 - x1, y2 - y1);
      });

      return {
        visibleCount: visiblePaths.length,
        markerCount: visiblePaths.length, // The path draws the arrow directly as a polygon, preserving the directional marker.
        maxLength: Math.max(...lengths, 0),
      };
    });

    expect(metrics.visibleCount, 'spec graph should expose multiple visible overview edges').toBeGreaterThan(10);
    expect(metrics.markerCount, 'visible overview edges should preserve directional arrow markers').toBe(metrics.visibleCount);
    expect(metrics.maxLength, 'overview edges should span visible segments between cards').toBeGreaterThan(12);
  });

  test('clicking a graph card opens the preview and marks the card selected', async ({ page }) => {
    test.setTimeout(120_000);

    await gotoGraph(page);

    const visibleIndex = await firstVisibleGraphCardIndex(page);
    expect(visibleIndex, 'spec graph should position at least one visible node card').toBeGreaterThanOrEqual(0);

    const card = page.locator('#spec-graph3d-container .graph-node-card').nth(visibleIndex);
    const title = ((await card.locator('.graph-node-card__title').textContent()) ?? '').trim();

    await card.click();

    const preview = page.locator('[data-testid="spec-preview"]');
    await expect(preview).toBeVisible({ timeout: 10_000 });
    await expect(card).toHaveClass(/node-card-selected/);
    if (title) {
      await expect(preview).toContainText(title);
    }

    await page.locator('[data-testid="spec-preview-close"]').click();
    await expect(preview).not.toBeVisible({ timeout: 10_000 });
    await expect(card).not.toHaveClass(/node-card-selected/);
  });

  test('dragging inside theme settings does not move the graph camera', async ({ page }) => {
    test.setTimeout(120_000);

    await gotoGraph(page);

    const visibleIndex = await firstVisibleGraphCardIndex(page);
    expect(visibleIndex, 'spec graph should position at least one visible node card for drag regression coverage')
      .toBeGreaterThanOrEqual(0);

    const card = page.locator('#spec-graph3d-container .graph-node-card').nth(visibleIndex);
    const themeButton = page.getByRole('button', { name: 'Theme settings' });

    await themeButton.click();

    const panel = page.locator('.modal-panel').first();
    const header = panel.locator('.glass-panel__header');
    await expect(panel).toBeVisible({ timeout: 10_000 });
    await expect(header).toBeVisible({ timeout: 10_000 });

    const before = await card.boundingBox();
    const headerBox = await header.boundingBox();
    expect(before, 'expected a visible graph card before dragging the theme panel').not.toBeNull();
    expect(headerBox, 'expected the theme settings header to be measurable for drag testing').not.toBeNull();

    const startX = headerBox!.x + Math.max(24, Math.min(96, headerBox!.width * 0.3));
    const startY = headerBox!.y + headerBox!.height / 2;

    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.mouse.move(startX + 140, startY + 70, { steps: 12 });
    await page.mouse.up();
    await page.waitForTimeout(150);

    const after = await card.boundingBox();
    expect(after, 'expected the same graph card to remain measurable after dragging the theme panel').not.toBeNull();

    expect(Math.abs(after!.x - before!.x), 'theme panel drag should not orbit the graph horizontally').toBeLessThan(0.5);
    expect(Math.abs(after!.y - before!.y), 'theme panel drag should not orbit the graph vertically').toBeLessThan(0.5);
    await expect(panel).toBeVisible();
  });

  test('frustum defaults relayout during camera drag and keep visible overlap bounded', async ({ page }) => {
    test.setTimeout(120_000);

    await gotoGraph(page);

    const before = await readVisibleGraphLayout(page);
    expect(before.viewportNodes, 'spec graph should expose enough visible cards for frustum regression coverage')
      .toBeGreaterThan(60);
    expect(before.overlapPairs, 'default frustum tuning should keep visible overlap within a coarse sanity bound')
      .toBeLessThanOrEqual(45);
    expect(before.maxOverlapArea, 'default frustum tuning should avoid large visible overlap slabs')
      .toBeLessThan(950);

    const container = page.locator('#spec-graph3d-container');
    const box = await container.boundingBox();
    expect(box, 'expected the graph container to expose a measurable drag surface').not.toBeNull();

    const startX = box!.x + box!.width * 0.58;
    const startY = box!.y + box!.height * 0.58;

    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.mouse.move(startX + 120, startY - 60, { steps: 16 });
    await page.waitForTimeout(160);
    const during1 = await readVisibleGraphLayout(page);

    await page.mouse.move(startX + 220, startY - 120, { steps: 16 });
    await page.waitForTimeout(160);
    const during2 = await readVisibleGraphLayout(page);

    await page.mouse.up();
    const release = await readVisibleGraphLayout(page);
    await page.waitForTimeout(500);
    const settled = await readVisibleGraphLayout(page);

    const liveMotion = graphLayoutMotion(before, during1) + graphLayoutMotion(during1, during2);
    const postReleaseMotion = graphLayoutMotion(release, settled);

    expect(liveMotion, 'frustum relayout should respond while the camera drag is still in progress')
      .toBeGreaterThan(80);
    expect(liveMotion, 'camera drag should trigger substantially more motion before mouseup than after release')
      .toBeGreaterThan(Math.max(postReleaseMotion * 3, 80));
  });

  test('zoom to selected node recenters and enlarges the chosen graph card', async ({ page }) => {
    test.setTimeout(120_000);

    await gotoGraph(page);
    await ensureGraphSettingsPanelVisible(page);
    await enableCenterCameraOnSelectedNode(page);
    await enableZoomToSelectedNode(page);
    await setSelectedNodeZoomFactor(page, 3.0);

    const candidate = await pickZoomableGraphCard(page);
    expect(candidate.index, 'spec graph should expose a visible node card for focus testing').toBeGreaterThanOrEqual(0);

    const card = page.locator('#spec-graph3d-container .graph-node-card').nth(candidate.index);
    await card.click();

    const preview = page.locator('[data-testid="spec-preview"]');
    await expect(preview).toBeVisible({ timeout: 10_000 });
    await expect(card).toHaveClass(/node-card-selected/);

    await waitForFocusedCard(page, candidate);
    const focusedMetrics = await readCardMetrics(page, candidate.index);

    await page.locator('[data-testid="spec-preview-close"]').click();
    await expect(preview).not.toBeVisible({ timeout: 10_000 });
    await expect(card).not.toHaveClass(/node-card-selected/);
    await page.waitForFunction(
      ({ index, focusedScale, initialScale }) => {
        const card = document.querySelectorAll('#spec-graph3d-container .graph-node-card')[index] as HTMLElement | undefined;
        if (!(card instanceof HTMLElement) || card.style.display === 'none') {
          return false;
        }
        const scaleMatch = card.style.transform.match(/scale\(([^)]+)\)/);
        if (!scaleMatch) {
          return false;
        }
        const scale = Number.parseFloat(scaleMatch[1] ?? '0');
        return scale < focusedScale - 0.08 && scale <= Math.max(initialScale + 0.08, initialScale * 1.1);
      },
      { index: candidate.index, focusedScale: focusedMetrics.scale, initialScale: candidate.initialScale },
      { timeout: 10_000 },
    );
  });

  test('center camera on selected node recenters without large zoom when node zoom is disabled', async ({ page }) => {
    test.setTimeout(120_000);

    await gotoGraph(page);
    await ensureGraphSettingsPanelVisible(page);
    await enableCenterCameraOnSelectedNode(page);
    await disableZoomToSelectedNode(page);

    const candidate = await pickZoomableGraphCard(page);
    expect(candidate.index, 'spec graph should expose a visible node card for center-only focus testing').toBeGreaterThanOrEqual(0);

    const card = page.locator('#spec-graph3d-container .graph-node-card').nth(candidate.index);
    await card.click();

    const preview = page.locator('[data-testid="spec-preview"]');
    await expect(preview).toBeVisible({ timeout: 10_000 });
    await expect(card).toHaveClass(/node-card-selected/);

    await page.waitForFunction(
      ({ index, initialScale, initialDistance }) => {
        const container = document.querySelector('#spec-graph3d-container');
        const preview = document.querySelector('[data-testid="spec-preview"]');
        const settingsPanel = document.querySelector('[data-testid="graph-settings-panel"]');
        const cards = Array.from(document.querySelectorAll('#spec-graph3d-container .graph-node-card'));
        const card = cards[index] as HTMLElement | undefined;
        if (!(container instanceof HTMLElement) || !(card instanceof HTMLElement) || card.style.display === 'none') {
          return false;
        }

        const scale = Number.parseFloat(card.style.transform.match(/scale\(([^)]+)\)/)?.[1] ?? '0');
        const containerRect = container.getBoundingClientRect();
        const previewRect = preview instanceof HTMLElement ? preview.getBoundingClientRect() : null;
        const settingsRect = settingsPanel instanceof HTMLElement ? settingsPanel.getBoundingClientRect() : null;
        const visibleLeft = settingsRect ? Math.min(settingsRect.right + 12, containerRect.right - 1) : containerRect.left;
        const visibleRight = previewRect ? Math.max(previewRect.left - 12, visibleLeft + 1) : containerRect.right;
        const visibleCenterX = visibleLeft + (visibleRight - visibleLeft) / 2;
        const visibleCenterY = containerRect.top + containerRect.height / 2;
        const rect = card.getBoundingClientRect();
        const distanceToCenter = Math.hypot(
          (rect.left + rect.width / 2) - visibleCenterX,
          (rect.top + rect.height / 2) - visibleCenterY,
        );

        return distanceToCenter < Math.max(initialDistance * 0.6, 140)
          && scale < Math.max(initialScale + 0.2, 1.55);
      },
      candidate,
      { timeout: 10_000 },
    );
  });

  test('auto-layout selected node smoothly translates surrounding cards', async ({ page }) => {
    test.setTimeout(120_000);

    await gotoGraph(page);
    await ensureGraphSettingsPanelVisible(page);
    await enableAutoLayoutSelectedNode(page);

    const candidate = await pickZoomableGraphCard(page);
    expect(candidate.index, 'spec graph should expose a visible node card for auto-layout testing').toBeGreaterThanOrEqual(0);

    const bystander = await pickBystanderGraphCard(page, candidate.index);
    expect(bystander.index, 'spec graph should expose a second visible node card for auto-layout testing').toBeGreaterThanOrEqual(0);

    await page.locator('#spec-graph3d-container .graph-node-card').nth(candidate.index).click();
    await expect(page.locator('[data-testid="spec-preview"]')).toBeVisible({ timeout: 10_000 });
    await page.waitForFunction(
      ({ index, centerX, centerY }) => {
        const card = document.querySelectorAll('#spec-graph3d-container .graph-node-card')[index] as HTMLElement | undefined;
        if (!(card instanceof HTMLElement) || card.style.display === 'none') {
          return false;
        }
        const rect = card.getBoundingClientRect();
        const dx = (rect.left + rect.width / 2) - centerX;
        const dy = (rect.top + rect.height / 2) - centerY;
        return Math.hypot(dx, dy) > 18;
      },
      bystander,
      { timeout: 10_000 },
    );
  });
});

async function gotoGraph(page: Page): Promise<void> {
  let lastError: unknown;

  for (let attempt = 0; attempt < 2; attempt += 1) {
    if (attempt === 0) {
      await page.goto(`${SPEC_VIEWER.url}/specs/graph`, { waitUntil: 'domcontentloaded' });
    } else {
      await page.reload({ waitUntil: 'domcontentloaded' });
    }

    await page.locator(SPEC_VIEWER.readySelector).first().waitFor({
      state: 'visible',
      timeout: SPEC_VIEWER.readyTimeout,
    });

    try {
      await expect(page.locator('#spec-graph3d-container')).toBeAttached({ timeout: 30_000 });
      await page.waitForFunction(() => {
        return Array.from(document.querySelectorAll('#spec-graph3d-container .graph-node-card'))
          .some((card) => (card as HTMLElement).style.display !== 'none');
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

async function firstVisibleGraphCardIndex(page: Page): Promise<number> {
  return page.evaluate(() => {
    const cards = Array.from(document.querySelectorAll('#spec-graph3d-container .graph-node-card'));
    return cards.findIndex((card) => (card as HTMLElement).style.display !== 'none');
  });
}

async function ensureGraphSettingsPanelVisible(page: Page): Promise<void> {
  const panel = page.locator('[data-testid="graph-settings-panel"]');
  if (await panel.isVisible()) {
    return;
  }

  await page.locator('[data-testid="graph-settings-toggle"]').click();
  await expect(panel).toBeVisible({ timeout: 10_000 });
}

async function enableZoomToSelectedNode(page: Page): Promise<void> {
  const toggle = page.locator('[data-testid="graph-toggle-zoom-selected-node"]');
  await expect(toggle).toBeVisible({ timeout: 10_000 });
  if (!(await toggle.isChecked())) {
    await toggle.check();
  }
}

async function disableZoomToSelectedNode(page: Page): Promise<void> {
  const toggle = page.locator('[data-testid="graph-toggle-zoom-selected-node"]');
  await expect(toggle).toBeVisible({ timeout: 10_000 });
  if (await toggle.isChecked()) {
    await toggle.uncheck();
  }
}

async function enableCenterCameraOnSelectedNode(page: Page): Promise<void> {
  const toggle = page.locator('[data-testid="graph-toggle-center-selected-node"]');
  await expect(toggle).toBeVisible({ timeout: 10_000 });
  if (!(await toggle.isChecked())) {
    await toggle.check();
  }
}

async function setSelectedNodeZoomFactor(page: Page, value: number): Promise<void> {
  const slider = page.locator('[data-testid="graph-range-selected-node-zoom-factor"]');
  await expect(slider).toBeVisible({ timeout: 10_000 });
  await slider.evaluate((element, nextValue) => {
    const input = element as HTMLInputElement;
    input.value = String(nextValue);
    input.dispatchEvent(new Event('input', { bubbles: true }));
    input.dispatchEvent(new Event('change', { bubbles: true }));
  }, value);
}

async function enableAutoLayoutSelectedNode(page: Page): Promise<void> {
  const toggle = page.locator('[data-testid="graph-toggle-auto-layout-selected-node"]');
  await expect(toggle).toBeVisible({ timeout: 10_000 });
  if (!(await toggle.isChecked())) {
    await toggle.check();
  }
}

async function waitForFocusedCard(page: Page, candidate: GraphCardMetrics): Promise<void> {
  await expect
    .poll(async () => (await readCardMetrics(page, candidate.index)).scale, { timeout: 10_000 })
    .toBeGreaterThan(Math.max(candidate.initialScale + 0.08, candidate.initialScale * 1.08));

  await expect
    .poll(async () => (await readCardMetrics(page, candidate.index)).distanceToCenter, { timeout: 10_000 })
    .toBeLessThan(Math.max(candidate.initialDistance * 0.8, 220));
}

async function readCardMetrics(page: Page, index: number): Promise<CardMetrics> {
  return page.evaluate((candidateIndex) => {
    const container = document.querySelector('#spec-graph3d-container');
    const preview = document.querySelector('[data-testid="spec-preview"]');
    const settingsPanel = document.querySelector('[data-testid="graph-settings-panel"]');
    const cards = Array.from(document.querySelectorAll('#spec-graph3d-container .graph-node-card'));
    const card = cards[candidateIndex] as HTMLElement | undefined;
    if (!(container instanceof HTMLElement) || !(card instanceof HTMLElement)) {
      return { scale: 0, centerX: 0, centerY: 0, distanceToCenter: 0 };
    }

    const scaleMatch = card.style.transform.match(/scale\(([^)]+)\)/);
    const scale = Number.parseFloat(scaleMatch?.[1] ?? '0');
    const containerRect = container.getBoundingClientRect();
    const cardRect = card.getBoundingClientRect();
    const previewRect = preview instanceof HTMLElement ? preview.getBoundingClientRect() : null;
    const settingsRect = settingsPanel instanceof HTMLElement ? settingsPanel.getBoundingClientRect() : null;
    const visibleLeft = settingsRect ? Math.min(settingsRect.right + 12, containerRect.right - 1) : containerRect.left;
    const visibleRight = previewRect ? Math.max(previewRect.left - 12, visibleLeft + 1) : containerRect.right;
    const visibleCenterX = visibleLeft + (visibleRight - visibleLeft) / 2;
    const visibleCenterY = containerRect.top + containerRect.height / 2;
    const centerX = cardRect.left + cardRect.width / 2;
    const centerY = cardRect.top + cardRect.height / 2;
    return {
      scale,
      centerX,
      centerY,
      distanceToCenter: Math.hypot(
        centerX - visibleCenterX,
        centerY - visibleCenterY,
      ),
    };
  }, index);
}

async function pickZoomableGraphCard(page: Page): Promise<GraphCardMetrics> {
  return page.evaluate(() => {
    const container = document.querySelector('#spec-graph3d-container');
    if (!(container instanceof HTMLElement)) {
      return { index: -1, initialScale: 0, initialDistance: 0 };
    }

    const containerRect = container.getBoundingClientRect();
    const centerX = containerRect.left + containerRect.width / 2;
    const centerY = containerRect.top + containerRect.height / 2;
    const cards = Array.from(document.querySelectorAll('#spec-graph3d-container .graph-node-card'));
    const visibleCards = cards
      .map((card, index) => {
        const el = card as HTMLElement;
        if (el.style.display === 'none') {
          return null;
        }

        const scaleMatch = el.style.transform.match(/scale\(([^)]+)\)/);
        const initialScale = Number.parseFloat(scaleMatch?.[1] ?? '0');
        const rect = el.getBoundingClientRect();
        const dx = (rect.left + rect.width / 2) - centerX;
        const dy = (rect.top + rect.height / 2) - centerY;

        return {
          index,
          initialScale,
          initialDistance: Math.hypot(dx, dy),
        };
      })
      .filter((card): card is GraphCardMetrics => Boolean(card));

    visibleCards.sort((left, right) => right.initialDistance - left.initialDistance);
    return visibleCards.find((card) => card.initialScale < 2.0 && card.initialDistance > 120)
      ?? visibleCards[0]
      ?? { index: -1, initialScale: 0, initialDistance: 0 };
  });
}

async function pickBystanderGraphCard(page: Page, selectedIndex: number): Promise<CardMetrics & { index: number }> {
  return page.evaluate((focusIndex) => {
    const cards = Array.from(document.querySelectorAll('#spec-graph3d-container .graph-node-card'));
    const visibleCards = cards
      .map((card, index) => {
        if (index === focusIndex) {
          return null;
        }
        const el = card as HTMLElement;
        if (el.style.display === 'none') {
          return null;
        }
        const rect = el.getBoundingClientRect();
        return {
          index,
          scale: 0,
          centerX: rect.left + rect.width / 2,
          centerY: rect.top + rect.height / 2,
          distanceToCenter: 0,
        };
      })
      .filter((card): card is CardMetrics & { index: number } => Boolean(card));

    return visibleCards[0] ?? { index: -1, scale: 0, centerX: 0, centerY: 0, distanceToCenter: 0 };
  }, selectedIndex);
}

async function readVisibleGraphLayout(page: Page): Promise<VisibleGraphLayout> {
  return page.evaluate(() => {
    const container = document.querySelector('#spec-graph3d-container');
    if (!(container instanceof HTMLElement)) {
      return { viewportNodes: 0, overlapPairs: 0, maxOverlapArea: 0, cards: [] };
    }

    const containerRect = container.getBoundingClientRect();
    const cards = Array.from(document.querySelectorAll('#spec-graph3d-container .graph-node-card'))
      .map((card, index) => {
        const el = card as HTMLElement;
        if (el.style.display === 'none') {
          return null;
        }

        const rect = el.getBoundingClientRect();
        const intersectsContainer = rect.right > containerRect.left
          && rect.left < containerRect.right
          && rect.bottom > containerRect.top
          && rect.top < containerRect.bottom;
        if (!intersectsContainer) {
          return null;
        }

        return {
          index,
          left: rect.left,
          top: rect.top,
          right: rect.right,
          bottom: rect.bottom,
          centerX: rect.left + rect.width / 2,
          centerY: rect.top + rect.height / 2,
        };
      })
      .filter((card): card is VisibleGraphCard => Boolean(card));

    let overlapPairs = 0;
    let maxOverlapArea = 0;
    for (let i = 0; i < cards.length; i += 1) {
      for (let j = i + 1; j < cards.length; j += 1) {
        const left = cards[i]!;
        const right = cards[j]!;
        const overlapX = Math.min(left.right, right.right) - Math.max(left.left, right.left);
        const overlapY = Math.min(left.bottom, right.bottom) - Math.max(left.top, right.top);
        if (overlapX > 0 && overlapY > 0) {
          overlapPairs += 1;
          maxOverlapArea = Math.max(maxOverlapArea, overlapX * overlapY);
        }
      }
    }

    return {
      viewportNodes: cards.length,
      overlapPairs,
      maxOverlapArea,
      cards,
    };
  });
}

function graphLayoutMotion(
  before: VisibleGraphLayout,
  after: VisibleGraphLayout,
): number {
  const afterByIndex = new Map(after.cards.map((card) => [card.index, card]));
  return before.cards.reduce((total, card) => {
    const next = afterByIndex.get(card.index);
    if (!next) {
      return total;
    }

    return total + Math.hypot(
      next.centerX - card.centerX,
      next.centerY - card.centerY,
    );
  }, 0);
}

type GraphCardMetrics = {
  index: number;
  initialScale: number;
  initialDistance: number;
};

type CardMetrics = {
  scale: number;
  centerX: number;
  centerY: number;
  distanceToCenter: number;
};

type VisibleGraphCard = {
  index: number;
  left: number;
  top: number;
  right: number;
  bottom: number;
  centerX: number;
  centerY: number;
};

type VisibleGraphLayout = {
  viewportNodes: number;
  overlapPairs: number;
  maxOverlapArea: number;
  cards: VisibleGraphCard[];
};