import { test, expect } from '@playwright/test';

test.describe('Homepage', () => {
  test('should load the homepage and display welcome message', async ({ page }) => {
    await page.goto('/');
    
    // Check page title
    await expect(page).toHaveTitle(/Groups App/);
    
    // Check welcome message
    await expect(page.locator('h2')).toContainText('Welcome to Groups App');
    await expect(page.locator('p').first()).toContainText('This is a simple application to manage groups.');
    
    // Check that the counter div exists (even if HTMX hasn't loaded the content yet)
    const counterDiv = page.locator('div[hx-get="/api/hello"]');
    await expect(counterDiv).toBeVisible();
    
    // Check the CTA button
    const groupsLink = page.locator('a:has-text("View All Groups")');
    await expect(groupsLink).toBeVisible();
    await expect(groupsLink).toHaveAttribute('href', '/groups');
  });

  test('should have correct page structure', async ({ page }) => {
    await page.goto('/');
    
    // Check navigation exists
    await expect(page.locator('nav')).toBeVisible();
    
    // Check main content container exists
    await expect(page.locator('main')).toBeVisible();
    
    // Check footer exists
    await expect(page.locator('footer')).toBeVisible();
  });

  test('API endpoint should return counter data', async ({ page, request }) => {
    // Test the API endpoint directly
    const response = await request.get('/api/hello');
    await expect(response).toBeOK();
    
    const text = await response.text();
    expect(text).toContain('Request number');
    expect(text).toMatch(/<strong>\d+<\/strong>/);
  });
});