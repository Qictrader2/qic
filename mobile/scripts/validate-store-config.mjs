#!/usr/bin/env node
/**
 * #557 / #558 — store-submission config guard.
 *
 * The Expo shell's store readiness lives entirely in `app.json`, `eas.json`
 * and `src/config.ts`. Every one of the checks below maps to a concrete
 * rejection or outage we would otherwise only discover from a review
 * rejection email or, worse, from users landing on the wrong environment.
 *
 * Deliberately dependency-free so it runs in CI without installing the
 * React Native toolchain: `node scripts/validate-store-config.mjs`.
 */

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const read = (p) => readFileSync(join(root, p), "utf8");
const readJson = (p) => JSON.parse(read(p));

const failures = [];
const check = (label, condition, detail) => {
  if (!condition) failures.push(`${label}\n    ${detail}`);
};

const app = readJson("app.json").expo;
const eas = readJson("eas.json");
const configSrc = read("src/config.ts");

// ── identity ────────────────────────────────────────────────────────────────

check(
  "bundle identifier and Android package must match",
  app.ios.bundleIdentifier === app.android.package,
  `ios.bundleIdentifier=${app.ios.bundleIdentifier} android.package=${app.android.package}. ` +
    "They key deep links, Play App Signing and analytics; a mismatch is very " +
    "expensive to change after the first release."
);

check(
  "app version must be set",
  typeof app.version === "string" && /^\d+\.\d+\.\d+$/.test(app.version),
  `expo.version=${app.version} — Apple and Play both require a semver marketing version.`
);

// ── permissions must be justified on both platforms ─────────────────────────

// Apple rejects a camera/mic permission with no usage string, and Play's Data
// Safety form has to match the manifest. Keep the two sides in lockstep.
const permissionUsageStrings = {
  "android.permission.CAMERA": "NSCameraUsageDescription",
  "android.permission.RECORD_AUDIO": "NSMicrophoneUsageDescription",
};

for (const [permission, plistKey] of Object.entries(permissionUsageStrings)) {
  if (!app.android.permissions.includes(permission)) continue;
  const usage = app.ios.infoPlist?.[plistKey];
  check(
    `${permission} needs a matching iOS usage string`,
    typeof usage === "string" && usage.trim().length > 0,
    `${plistKey} is missing or empty. Apple rejects builds that request the ` +
      "capability without explaining why."
  );
}

// ── iOS privacy manifest (mandatory since 2024) ─────────────────────────────

const privacy = app.ios.privacyManifests;
check(
  "iOS privacy manifest must be present",
  privacy !== undefined,
  "ios.privacyManifests is required for App Store submission since spring 2024."
);

if (privacy) {
  check(
    "the shell must not declare tracking",
    privacy.NSPrivacyTracking === false,
    `NSPrivacyTracking=${privacy.NSPrivacyTracking}. The WebView shell embeds no ` +
      "ad SDK; declaring tracking would force an App Tracking Transparency prompt."
  );

  const apis = privacy.NSPrivacyAccessedAPITypes ?? [];
  check(
    "required-reason APIs must be declared",
    apis.length > 0,
    "NSPrivacyAccessedAPITypes is empty. React Native touches UserDefaults and " +
      "file timestamps, both of which are required-reason APIs."
  );

  for (const entry of apis) {
    check(
      `${entry.NSPrivacyAccessedAPIType} needs at least one reason code`,
      Array.isArray(entry.NSPrivacyAccessedAPITypeReasons) &&
        entry.NSPrivacyAccessedAPITypeReasons.length > 0,
      "An accessed-API entry with no reason code fails App Store validation."
    );
  }
}

// ── EAS build profiles ──────────────────────────────────────────────────────

const production = eas.build?.production;
check("eas.json must define a production profile", production !== undefined,
  "Without it there is no reproducible store build.");

if (production) {
  check(
    "the production Android build must be an app bundle",
    production.android?.buildType === "app-bundle",
    `android.buildType=${production.android?.buildType}. Play has required AAB ` +
      "rather than APK for new apps since August 2021."
  );

  check(
    "the production build must auto-increment its version",
    production.autoIncrement === true,
    "Play and App Store Connect both reject a build whose version code has " +
      "already been uploaded; manual bumps get forgotten."
  );

  check(
    "the production build must target the production web app",
    production.env?.EXPO_PUBLIC_WEB_APP_URL === "https://www.qictrader.com",
    `production env URL is ${production.env?.EXPO_PUBLIC_WEB_APP_URL}. Shipping a ` +
      "store build pointed at staging would expose test data to real users."
  );
}

for (const name of ["development", "preview"]) {
  const profile = eas.build?.[name];
  if (!profile) continue;
  check(
    `the ${name} profile must not point at production`,
    profile.env?.EXPO_PUBLIC_WEB_APP_URL !== "https://www.qictrader.com",
    "Internal builds hitting production risk real trades from test devices."
  );
}

// ── environment default stays staging until JP flips it ─────────────────────

check(
  "the in-code default web app URL must remain staging",
  /\?\?\s*"https:\/\/staging\.qictrader\.com"/.test(configSrc),
  "src/config.ts no longer defaults to staging. Production targeting belongs in " +
    "the EAS production profile; changing the code default makes every local " +
    "and preview build hit production. Flipping this needs JP sign-off."
);

// ── deep links ──────────────────────────────────────────────────────────────

const PRODUCTION_HOST = "www.qictrader.com";
// Exact hosts only. A wildcard entry such as `*.qictrader.com` would hand every
// subdomain the right to open the app, so the allowlist is enumerated rather
// than pattern-matched, and anything outside it fails below.
const ALLOWED_DEEP_LINK_HOSTS = new Set([PRODUCTION_HOST, "staging.qictrader.com"]);

const filters = app.android.intentFilters ?? [];
const hosts = filters.flatMap((f) => (f.data ?? []).map((d) => d.host));

check(
  "production host must be registered for deep links",
  hosts.some((h) => h === PRODUCTION_HOST),
  `registered hosts: ${hosts.join(", ") || "none"}`
);

const unexpectedHosts = hosts.filter((h) => !ALLOWED_DEEP_LINK_HOSTS.has(h));
check(
  "every deep-link host must be an exact, allowlisted hostname",
  unexpectedHosts.length === 0,
  `unexpected host(s): ${unexpectedHosts.join(", ")}. Android matches intent-filter ` +
    "hosts as patterns, so a wildcard or a lookalike domain lets a host we do not " +
    "control open the app and receive the link. List exact hostnames only."
);

// `autoVerify: true` without a served assetlinks.json leaves links opening in
// the browser and hides the misconfiguration, so the two must move together.
const autoVerifying = filters.some((f) => f.autoVerify === true);
check(
  "autoVerify must stay off until assetlinks.json is served",
  autoVerifying === false,
  "autoVerify is enabled, which only works once /.well-known/assetlinks.json is " +
    "served from www.qictrader.com with the release signing certificate " +
    "fingerprint. That needs the upload keystore, which does not exist yet " +
    "(#558). Enable both together."
);

// ── report ──────────────────────────────────────────────────────────────────

if (failures.length > 0) {
  console.error(`\nStore config validation FAILED (${failures.length}):\n`);
  failures.forEach((f, i) => console.error(`  ${i + 1}. ${f}\n`));
  process.exit(1);
}

console.log("Store config validation passed.");
