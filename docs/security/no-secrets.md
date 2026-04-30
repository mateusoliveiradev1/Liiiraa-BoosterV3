# No-Secret Guardrails

Liiiraa Booster treats environment examples as documentation, not storage for
real values. The repository should contain names, safe defaults, and empty
placeholders only.

## Boundaries

- API secrets may exist only in ignored local files or protected deployment
  secret stores.
- Desktop and web environment variables must stay public and prefixed for the
  frontend runtime.
- Neon credentials must never be exposed to desktop, web, telemetry, or release
  artifacts.
- Update signing material must stay outside the repository and outside app
  runtime environment files.
- Crash reporting and telemetry defaults must remain opt-in or off until a
  consent gate enables them.

## Review Checklist

- Committed env files end in `.env.example`.
- Secret-valued variables in examples are empty.
- Local `.env`, `.env.local`, certificates, keys, and secret directories remain
  ignored.
- API logs use request IDs and redacted errors rather than raw credentials.
- Migrations use reviewed direct database connections only when required.

## Local Secret Scan

Run this fallback scan before committing env or docs changes:

```powershell
rg -n --hidden --glob '!node_modules/**' --glob '!.git/**' --glob '!pnpm-lock.yaml' "(AKIA[0-9A-Z]{16}|-----BEGIN (RSA |OPENSSH |EC |DSA |)?PRIVATE KEY-----|ghp_[A-Za-z0-9_]{36,}|github_pat_[A-Za-z0-9_]+|xox[baprs]-[A-Za-z0-9-]+|sk-[A-Za-z0-9]{20,}|postgres(?:ql)?://[^\s<>]*:[^\s<>]*@)"
```

If the scan finds a real secret, rotate it before continuing and remove it from
the repository history according to the incident response process.
