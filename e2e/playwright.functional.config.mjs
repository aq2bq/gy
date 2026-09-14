import {defineConfig} from '@playwright/test';
import base from './playwright.config.mjs';

// CI retains the full gate even when it invokes the local default entry point.
export default defineConfig(base, {
  testIgnore: process.env.CI ? [] : ['**/performance.spec.mjs'],
});
