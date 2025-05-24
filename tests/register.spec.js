import { test, expect } from '@playwright/test';

test.describe('Registration Page', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the registration page before each test
    await page.goto('http://localhost:8080/register');
  });

  test('should display registration form', async ({ page }) => {
    // Check that the registration page loads correctly
    await expect(page.locator('h2')).toHaveText('Register');

    // Check form elements are present
    await expect(page.locator('label[for="email"]')).toHaveText('Email:');
    await expect(page.locator('input#email')).toBeVisible();
    await expect(page.locator('input#email')).toHaveAttribute('type', 'email');

    await expect(page.locator('label[for="password"]')).toHaveText('Password:');
    await expect(page.locator('input#password')).toBeVisible();
    await expect(page.locator('input#password')).toHaveAttribute('type', 'password');
    await expect(page.locator('input#password')).toHaveAttribute('minlength', '8');

    await expect(page.locator('button[type="submit"]')).toHaveText('Register');
    await expect(page.locator('.form-links a[href="/login"]')).toBeVisible();
  });

  test('should register new user successfully', async ({ page }) => {
    // Generate unique email for test
    const uniqueEmail = `test${Date.now()}@example.com`;

    // Fill in registration form
    await page.fill('#email', uniqueEmail);
    await page.fill('#password', 'password123');

    // Submit the form
    await page.click('button[type="submit"]');

    // Wait for success message
    await expect(page.locator('.alert-success')).toBeVisible();
    await expect(page.locator('.alert-success')).toHaveText('Registration successful! Please login.');

    // Wait for redirect to login page
    await page.waitForTimeout(2000);
    await expect(page).toHaveURL('http://localhost:8080/login');
  });

  test('should show error for duplicate email', async ({ page }) => {
    // First register a user
    const testEmail = 'duplicate@example.com';
    await page.fill('#email', testEmail);
    await page.fill('#password', 'password123');
    await page.click('button[type="submit"]');

    // Wait for success and redirect
    await expect(page.locator('.alert-success')).toBeVisible();
    await page.waitForTimeout(2000);

    // Go back to registration page
    await page.goto('http://localhost:8080/register');

    // Try to register with the same email
    await page.fill('#email', testEmail);
    await page.fill('#password', 'password123');
    await page.click('button[type="submit"]');

    // Wait for error message
    await expect(page.locator('.alert-error')).toBeVisible();
    await expect(page.locator('.alert-error')).toHaveText('Email already registered');

    // Ensure we're still on the registration page
    await expect(page).toHaveURL('http://localhost:8080/register');
  });

  test('should validate email format', async ({ page }) => {
    // Fill in invalid email format
    await page.fill('#email', 'notanemail');
    await page.fill('#password', 'password123');

    // Try to submit
    await page.click('button[type="submit"]');

    // Browser should prevent submission due to invalid email
    const emailInput = page.locator('#email');
    const isInvalid = await emailInput.evaluate(el => !el.validity.valid);
    expect(isInvalid).toBeTruthy();
  });

  test('should validate password length', async ({ page }) => {
    // Fill in password that's too short
    await page.fill('#email', 'test@example.com');
    await page.fill('#password', 'short');

    // Try to submit
    await page.click('button[type="submit"]');

    // Browser should prevent submission due to short password
    const passwordInput = page.locator('#password');
    const isInvalid = await passwordInput.evaluate(el => !el.validity.valid);
    expect(isInvalid).toBeTruthy();
  });

  test('should handle empty form submission', async ({ page }) => {
    // Try to submit without filling any fields
    await page.click('button[type="submit"]');

    // Browser's built-in validation should prevent submission
    const emailInput = page.locator('#email');
    const isInvalid = await emailInput.evaluate(el => !el.validity.valid);
    expect(isInvalid).toBeTruthy();
  });

  test('htmz integration should work correctly', async ({ page }) => {
    // Check that form targets htmz
    const form = page.locator('form');
    await expect(form).toHaveAttribute('target', 'htmz');
    await expect(form).toHaveAttribute('action', '/auth/register#register-form');
  });

  test('should navigate between login and register', async ({ page }) => {
    // Click on login link
    await page.click('a[href="/login"]');

    // Should navigate to login page
    await expect(page).toHaveURL('http://localhost:8080/login');
    await expect(page.locator('h2')).toHaveText('Login');

    // Navigate back to register
    await page.click('a[href="/register"]');

    // Should be back on register page
    await expect(page).toHaveURL('http://localhost:8080/register');
    await expect(page.locator('h2')).toHaveText('Register');
  });
});