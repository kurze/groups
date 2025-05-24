import { test, expect } from '@playwright/test';

test.describe('Login Page', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the login page before each test
    await page.goto('http://localhost:8080/login');
  });

  test('should display login form', async ({ page }) => {
    // Check that the login page loads correctly
    await expect(page.locator('h2')).toHaveText('Login');

    // Check form elements are present
    await expect(page.locator('label[for="email"]')).toHaveText('Email:');
    await expect(page.locator('input#email')).toBeVisible();

    await expect(page.locator('label[for="password"]')).toHaveText('Password:');
    await expect(page.locator('input#password')).toBeVisible();

    await expect(page.locator('button[type="submit"]')).toHaveText('Login');
    await expect(page.locator('.form-links a[href="/register"]')).toBeVisible();
  });

  test('should show error message for invalid credentials', async ({ page }) => {
    // Fill in invalid credentials
    await page.fill('#email', 'wrong@example.com');
    await page.fill('#password', 'wrongpassword');

    // Submit the form
    await page.click('button[type="submit"]');

    // Wait for the error message to appear
    await expect(page.locator('.alert-error')).toBeVisible();
    await expect(page.locator('.alert-error')).toHaveText('Invalid email or password');

    // Ensure we're still on the login page
    await expect(page).toHaveURL('http://localhost:8080/login');
  });

  test('should login successfully with valid credentials', async ({ page }) => {
    // Fill in valid credentials
    await page.fill('#email', 'test@example.com');
    await page.fill('#password', 'password');

    // Submit the form
    await page.click('button[type="submit"]');

    // Wait for success message
    await expect(page.locator('.alert-success')).toBeVisible();
    await expect(page.locator('.alert-success')).toHaveText('Login successful!');

    // The redirect happens via JavaScript after 1.5 seconds
    // Wait a bit longer and then check the URL
    await page.waitForTimeout(2000);

    // Verify we're on the groups page
    await expect(page).toHaveURL('http://localhost:8080/groups');
    await expect(page.locator('h2')).toHaveText('Groups');
  });

  test('should handle empty form submission', async ({ page }) => {
    // Try to submit without filling any fields
    await page.click('button[type="submit"]');

    // Browser's built-in validation should prevent submission
    // Check that required fields show validation
    const emailInput = page.locator('#email');
    const isInvalid = await emailInput.evaluate(el => !el.validity.valid);
    expect(isInvalid).toBeTruthy();
  });

  test('should validate email format', async ({ page }) => {
    // Fill in invalid email format
    await page.fill('#email', 'notanemail');
    await page.fill('#password', 'somepassword');

    // Try to submit
    await page.click('button[type="submit"]');

    // Browser should prevent submission due to invalid email
    const emailInput = page.locator('#email');
    const isInvalid = await emailInput.evaluate(el => !el.validity.valid);
    expect(isInvalid).toBeTruthy();
  });

  test('htmz integration should work correctly', async ({ page }) => {
    // Check that htmz iframe is present
    const htmzIframe = page.locator('iframe[name="htmz"]');
    await expect(htmzIframe).toBeHidden(); // Should be hidden
    await expect(htmzIframe).toHaveAttribute('hidden', '');

    // Check that form targets htmz
    const form = page.locator('form');
    await expect(form).toHaveAttribute('target', 'htmz');
    await expect(form).toHaveAttribute('action', '/auth/login#login-form');
  });
});