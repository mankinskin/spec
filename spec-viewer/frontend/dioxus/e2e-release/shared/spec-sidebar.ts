import { expect, type Locator, type Page } from '@playwright/test';

type SpecListResponse = {
  items: SpecSummary[];
};

type SpecSummary = {
  id: string;
  slug?: string | null;
  title?: string | null;
  component?: string | null;
};

export type SidebarSpec = {
  id: string;
  slug: string;
  label: string;
  component: string;
};

export async function fetchFirstSidebarLeaf(page: Page): Promise<SidebarSpec> {
  const specs = await fetchSidebarSpecs(page);
  expect(specs.length, 'expected /api/specs to expose at least one sidebar leaf').toBeGreaterThan(0);
  return specs[0]!;
}

export async function fetchSidebarLeafPair(page: Page): Promise<{
  first: SidebarSpec;
  second: SidebarSpec;
}> {
  const specs = await fetchSidebarSpecs(page);

  for (let index = 0; index < specs.length; index += 1) {
    const first = specs[index]!;
    const second = specs.slice(index + 1).find((candidate) => candidate.component === first.component);
    if (second) {
      return { first, second };
    }
  }

  throw new Error('expected /api/specs to expose two sidebar leaves in the same component folder');
}

export async function clickSidebarSpec(page: Page, spec: SidebarSpec): Promise<void> {
  const row = sidebarRow(page, spec);
  if (!(await row.isVisible())) {
    const folderRow = page.locator('.tree-item-row[role="treeitem"]').filter({
      has: page.locator('.tree-label', { hasText: exactText(spec.component) }),
    }).first();

    await expect(folderRow).toBeVisible({ timeout: 20_000 });
    if ((await folderRow.getAttribute('aria-expanded')) !== 'true') {
      await folderRow.click();
      await expect(folderRow).toHaveAttribute('aria-expanded', 'true', { timeout: 10_000 });
    }
  }

  await expect(row).toBeVisible({ timeout: 10_000 });
  await row.click();
}

export function getCurrentSpecId(page: Page): string | null {
  return currentSpecIdFromUrl(page.url());
}

export function currentSpecIdFromUrl(rawUrl: string): string | null {
  const url = new URL(rawUrl);
  if (!url.pathname.startsWith('/specs/')) {
    return null;
  }

  const specId = url.pathname.slice('/specs/'.length);
  if (specId.length === 0 || specId === 'graph') {
    return null;
  }

  return decodeURIComponent(specId);
}

export function buildSpecDetailUrl(specId: string, view?: string): string {
  const encodedId = encodeURI(specId);
  const suffix = view ? `?view=${encodeURIComponent(view)}` : '';
  return `/specs/${encodedId}${suffix}`;
}

export function sidebarRow(page: Page, spec: SidebarSpec): Locator {
  return page.locator('.tree-item-row').filter({
    has: page.locator('.tree-label', { hasText: exactText(spec.label) }),
  }).first();
}

async function fetchSidebarSpecs(page: Page): Promise<SidebarSpec[]> {
  const response = await page.evaluate(async () => {
    const result = await fetch('/api/specs?limit=200');
    if (!result.ok) {
      throw new Error(`failed to fetch /api/specs: ${result.status}`);
    }

    return (await result.json()) as SpecListResponse;
  });

  const specs = response.items
    .map((item) => {
      const slug = item.slug?.trim() ?? '';
      const component = item.component?.trim() ?? '';
      const title = item.title?.trim() ?? '';
      return {
        id: item.id,
        slug,
        label: title || slug || 'Untitled',
        component,
      };
    })
    .filter((item): item is SidebarSpec => item.slug.length > 0 && item.component.length > 0);

  const labelCounts = new Map<string, number>();
  for (const spec of specs) {
    labelCounts.set(spec.label, (labelCounts.get(spec.label) ?? 0) + 1);
  }

  const uniqueLabelSpecs = specs.filter((spec) => labelCounts.get(spec.label) === 1);
  return uniqueLabelSpecs.length > 0 ? uniqueLabelSpecs : specs;
}

function exactText(value: string): RegExp {
  return new RegExp(`^${escapeRegExp(value)}$`);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}