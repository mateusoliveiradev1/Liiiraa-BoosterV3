export default {
  testDir: ".",
  testMatch: /.*\.spec\.mjs/,
  timeout: 30_000,
  expect: {
    timeout: 5_000
  },
  fullyParallel: false,
  use: {
    browserName: "chromium",
    trace: "retain-on-failure"
  }
};
