export default {
  testDir: ".",
  testMatch: /.*\.spec\.mjs/,
  timeout: 60_000,
  expect: {
    timeout: 5_000
  },
  fullyParallel: false,
  outputDir: "../test-results",
  use: {
    baseURL: "http://127.0.0.1:5174",
    browserName: "chromium",
    trace: "retain-on-failure"
  },
  webServer: {
    command: "pnpm --filter @liiiraa/desktop dev -- --host 127.0.0.1",
    reuseExistingServer: process.env.CI !== "true",
    timeout: 120_000,
    url: "http://127.0.0.1:5174"
  }
};
