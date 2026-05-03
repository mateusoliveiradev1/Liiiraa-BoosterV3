# Locale Decision

Date: 2026-05-03

## Decision

Use `en-US` as the default locale for the first implementation pass.

Reasoning:

- The current desktop app and shared UI fixtures are already English-heavy.
- The redesign should not block on final Portuguese or Spanish copy.
- Moving current English strings into locale keys first creates a stable typed
  boundary without changing screen behavior.
- `pt-BR` remains the product direction and should be filled as the first
  translation pass once the redesigned surfaces have stable keys.

## Supported Locales For This Change

- `en-US`: default and canonical source text during the transition.
- `pt-BR`: supported catalog placeholder, filled first after key stabilization.
- `es-ES`: supported catalog placeholder, filled after Portuguese coverage or
  alongside it where terms are already settled.

## Runtime Locale Selection Order

When locale selection is introduced, resolve the active locale in this order:

1. Persisted in-app locale preference, if supported and valid.
2. OS/browser locale when it exactly matches a supported locale.
3. OS/browser language family match, using the supported regional locale:
   `pt-* -> pt-BR`, `es-* -> es-ES`, `en-* -> en-US`.
4. Default locale: `en-US`.

## Key Lookup Fallback Order

For a requested locale key:

1. Active locale key.
2. `en-US` default locale key.
3. Development/test missing-key signal, while rendering a visible fallback such
   as `[missing:<key>]` in non-production contexts.
4. Production fallback to the key name only if both the active locale and
   `en-US` are missing, so the UI remains debuggable instead of blank.

## Formatting Fallback Order

Use the same resolved locale for number/date formatting. Existing hardcoded
formatting such as `toLocaleString("en-US")` in
`apps/desktop/src/routes/BenchmarksRoute.tsx` should move behind the locale
helper so benchmark row counts and metadata follow the active locale.

## First Pass Implementation Boundary

- Add typed catalogs for `en-US`, `pt-BR`, and `es-ES`.
- Store complete current English copy in `en-US`.
- Allow `pt-BR` and `es-ES` to use placeholders or inherited English until the
  translation pass lands.
- Centralize glossary-backed optimizer terms before route-by-route migration.
- Tests should expose missing keys in development/test and should include
  Portuguese and Spanish visual-fit checks after the first route migration.
