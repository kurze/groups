import { test, expect } from '@playwright/test';

test.describe('Session Management', () => {
  test('should show login/register links when not authenticated', async ({ page }) => {
    await page.goto('http://localhost:8080/');
    
    // Check auth menu shows login/register
    await expect(page.locator('.auth-menu a[href="/login"]')).toBeVisible();
    await expect(page.locator('.auth-menu a[href="/register"]')).toBeVisible();
    
    // Should not show logout or user info
    await expect(page.locator('.auth-menu a[href="/logout"]')).not.toBeVisible();
    await expect(page.locator('.user-info')).not.toBeVisible();
  });

  test('should show user info and logout when authenticated', async ({ page }) => {
    // First register and login
    await page.goto('http://localhost:8080/register');
    const testEmail = `session${Date.now()}@example.com`;
    const testPassword = 'password123';
    
    await page.fill('#email', testEmail);
    await page.fill('#password', testPassword);
    await page.click('button[type="submit"]');
    
    // Wait for registration and redirect to login
    await page.waitForTimeout(2000);
    
    // Login
    await page.fill('#email', testEmail);
    await page.fill('#password', testPassword);
    await page.click('button[type="submit"]');
    
    // Wait for login and redirect
    await page.waitForTimeout(2000);
    
    // Now check the auth menu
    await expect(page.locator('.user-info')).toBeVisible();
    await expect(page.locator('.user-info')).toContainText('Welcome');
    await expect(page.locator('.auth-menu a[href="/logout"]')).toBeVisible();
    
    // Should not show login/register
    await expect(page.locator('.auth-menu a[href="/login"]')).not.toBeVisible();
    await expect(page.locator('.auth-menu a[href="/register"]')).not.toBeVisible();
  });

  test('should maintain session across page navigation', async ({ page }) => {
    // Register and login
    await page.goto('http://localhost:8080/register');
    const testEmail = `nav${Date.now()}@example.com`;
    const testPassword = 'password123';
    
    await page.fill('#email', testEmail);
    await page.fill('#password', testPassword);
    await page.click('button[type="submit"]');
    await page.waitForTimeout(2000);
    
    await page.fill('#email', testEmail);
    await page.fill('#password', testPassword);
    await page.click('button[type="submit"]');
    await page.waitForTimeout(2000);
    
    // Navigate to home page
    await page.goto('http://localhost:8080/');
    await expect(page.locator('.user-info')).toBeVisible();
    
    // Navigate to groups page
    await page.goto('http://localhost:8080/groups');
    await expect(page.locator('.user-info')).toBeVisible();
    
    // Should see Create New Group button
    await expect(page.locator('a[href="/groups/new"]')).toBeVisible();
  });

  test('protected routes should redirect to login when not authenticated', async ({ page }) => {
    // Try to access protected route without authentication
    await page.goto('http://localhost:8080/groups/new');
    
    // Should redirect to login
    await expect(page).toHaveURL('http://localhost:8080/login');
    await expect(page.locator('h2')).toHaveText('Login');
  });

  test('logout should clear session and redirect', async ({ page }) => {
    // Register and login first
    await page.goto('http://localhost:8080/register');
    const testEmail = `logout${Date.now()}@example.com`;
    const testPassword = 'password123';
    
    await page.fill('#email', testEmail);
    await page.fill('#password', testPassword);
    await page.click('button[type="submit"]');
    await page.waitForTimeout(2000);
    
    await page.fill('#email', testEmail);
    await page.fill('#password', testPassword);
    await page.click('button[type="submit"]');
    await page.waitForTimeout(2000);
    
    // Verify logged in
    await expect(page.locator('.user-info')).toBeVisible();
    
    // Click logout
    await page.click('a[href="/logout"]');
    
    // Should redirect to home
    await expect(page).toHaveURL('http://localhost:8080/');
    
    // Should show login/register links again
    await expect(page.locator('.auth-menu a[href="/login"]')).toBeVisible();
    await expect(page.locator('.auth-menu a[href="/register"]')).toBeVisible();
    
    // Try to access protected route - should redirect to login
    await page.goto('http://localhost:8080/groups/new');
    await expect(page).toHaveURL('http://localhost:8080/login');
  });
});