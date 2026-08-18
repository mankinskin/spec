import { expect, test } from '@playwright/test';

import { SPEC_VIEWER, gotoAndWaitForViewer } from './shared/viewers';

async function fetchFirstSpecId(page: import('@playwright/test').Page): Promise<string> {
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

test.describe('spec-viewer — detail route query views', () => {
  test('selects and normalizes canonical detail views', async ({ page }) => {
    test.setTimeout(90_000);

    await gotoAndWaitForViewer(page, SPEC_VIEWER);
    const specId = await fetchFirstSpecId(page);
    const baseUrl = SPEC_VIEWER.url;

    await page.goto(`${baseUrl}/specs/${specId}?view=sections`, {
      waitUntil: 'domcontentloaded',
    });
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'direct detail links should preserve the canonical sections view query',
      })
      .toBe(`${baseUrl}/specs/${specId}?view=sections`);
    await expect(page.getByRole('button', { name: 'Sections' })).toHaveClass(/active/, {
      timeout: 10_000,
    });

    await page.getByRole('button', { name: 'CodeRefs' }).click();
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'tab clicks should update the detail route to the coderefs view',
      })
      .toBe(`${baseUrl}/specs/${specId}?view=coderefs`);
    await expect(page.getByRole('button', { name: 'CodeRefs' })).toHaveClass(/active/, {
      timeout: 10_000,
    });

    await page.goto(`${baseUrl}/specs/${specId}?view=code-refs`, {
      waitUntil: 'domcontentloaded',
    });
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'legacy code-refs query values should normalize to coderefs',
      })
      .toBe(`${baseUrl}/specs/${specId}?view=coderefs`);
    await expect(page.getByRole('button', { name: 'CodeRefs' })).toHaveClass(/active/, {
      timeout: 10_000,
    });

    await page.goto(`${baseUrl}/specs/${specId}?view=body`, {
      waitUntil: 'domcontentloaded',
    });
    await expect
      .poll(() => page.url(), {
        timeout: 15_000,
        message: 'body should normalize to the clean detail route without a query string',
      })
      .toBe(`${baseUrl}/specs/${specId}`);
    await expect(page.getByRole('button', { name: 'Body' })).toHaveClass(/active/, {
      timeout: 10_000,
    });
  });
});