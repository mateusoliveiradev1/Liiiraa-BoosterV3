# Windows Signing Path

Liiiraa Booster stable releases must prove two things before distribution: the
Windows executable/installer came from Liiiraa, and the updater metadata cannot
install tampered artifacts. Windows code signing and Tauri updater signing are
separate controls and both are required for stable releases.

## Release Identity

- Product name: `Liiiraa Booster`.
- Publisher display: `Liiiraa`.
- Public trust surface: the app should show `Signed by Liiiraa` with update and
  signing status wherever release trust information is presented.
- Stable release tags must be signed before any Windows artifact is promoted to
  stable.

## Certificate Handling

- Use an organization validation or extended validation code-signing
  certificate for public Windows distribution when the project is ready for
  paid release.
- Store certificate material only in the protected release environment or an
  HSM-backed signing service. Do not commit certificates, PFX files, private
  keys, passwords, or exported signing material.
- Limit access to signing secrets to release maintainers and protected
  environment approvals.
- Timestamp every Authenticode signature with a trusted RFC 3161 timestamp
  server so artifacts remain verifiable after certificate rotation or expiry.
- Record certificate subject, thumbprint, timestamp URL, signing operator, and
  release version in the release notes or internal release record.

## Artifact Signing

Every public Windows release artifact must be signed before publication:

- Desktop executable and bundled sidecars.
- Installer package, such as MSI or NSIS output.
- Uninstaller when generated as a separate binary by the installer toolchain.
- Any helper executable shipped with the desktop app, including elevated agents.

The signing step must run after the artifact is built and before upload to the
release channel. A stable release must block if `signtool verify` cannot verify
the expected publisher, certificate chain, and timestamp.

Example verification shape:

```powershell
signtool verify /pa /tw path\to\Liiiraa-Booster-Setup.exe
Get-AuthenticodeSignature path\to\Liiiraa-Booster-Setup.exe
```

## Installer Signing

Installer signing should make Windows prompts identify Liiiraa as the publisher
for the first install, upgrades, repair flows, and uninstall flows.

- Sign the final installer, not only the application executable inside it.
- Confirm the installer preserves signed payloads instead of repacking or
  modifying them after signing.
- Verify upgrades from beta to stable and stable to stable keep the expected
  publisher identity.
- Keep install mode intentional. Use passive install for updater-driven flows
  unless product testing chooses another mode.

## Tauri Updater Signing

Tauri updater signatures protect update integrity, while Authenticode protects
Windows publisher trust. Do not use one as a substitute for the other.

- Embed only the updater public key in the desktop app.
- Keep the updater private key in the protected release environment only.
- Produce signed updater artifacts and metadata for every update channel.
- Reject update metadata that is missing a valid signature.
- Use HTTPS update endpoints in production.
- Never enable insecure transport options for production update endpoints.

## SmartScreen Reputation

SmartScreen reputation is earned over time by consistent, signed distribution.
The release path should avoid resets and confusing publisher changes.

- Keep the publisher name stable as `Liiiraa`.
- Prefer a long-lived EV certificate when budget and company setup allow it;
  otherwise use OV signing and expect reputation to build gradually.
- Avoid changing certificate subjects, installer filenames, or download domains
  without a release note and rollout reason.
- Publish only signed stable artifacts from the official release domain or
  GitHub release page.
- Do not instruct users to disable SmartScreen. If Windows warns during early
  reputation building, explain the publisher, version, hash, and official
  download source instead.
- Track SmartScreen feedback during beta/stable rollout and pause promotion if
  warnings spike unexpectedly after a signing or distribution change.

## Stable Release Gate

Before promoting a Windows release to stable, confirm:

- Signed Git tag exists for the release version.
- Tauri updater artifacts and metadata are signed.
- Windows executable, installer, uninstaller, and helper executables are signed.
- Authenticode verification passes with the expected publisher and timestamp.
- Release artifacts contain no private signing keys, updater private keys,
  certificates, PFX files, or signing passwords.
- Release notes include Windows signing status, updater signature status, and
  artifact provenance status.
- The beta channel received privileged, Lab, updater, or signing-path changes
  before stable promotion.

## Review Checklist

- [ ] Certificate material is stored outside the repository and desktop app.
- [ ] Installer and shipped executables are Authenticode signed.
- [ ] Signatures include trusted timestamps.
- [ ] Tauri updater public/private key separation is documented and preserved.
- [ ] Stable release tags are signed.
- [ ] SmartScreen guidance avoids bypass instructions and names the official
      download source.
- [ ] Stable distribution blocks unsigned or unverifiable Windows artifacts.
