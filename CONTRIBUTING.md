# Contributing

This repository follows the OpenSpec task map in
`openspec/changes/build-liiiraa-boost-platform/tasks.md`. Work in one task-sized
commit, run the listed verification, then push the active branch.

## Local Git Identity

Use the GitHub identity that will sign and push the work. For the project owner,
the expected local identity is:

```bash
git config user.name "Mateus Oliveira"
git config user.email "warface01031999@gmail.com"
git config --list --show-origin
```

Use repository-local config when a machine has multiple GitHub accounts.

## Signed Commits

Commits and release tags are expected to be signed. Configure either SSH signing:

```bash
git config commit.gpgsign true
git config tag.gpgsign true
git config gpg.format ssh
git config user.signingkey ~/.ssh/<github-signing-key>.pub
```

Or GPG signing:

```bash
git config commit.gpgsign true
git config tag.gpgsign true
git config user.signingkey <gpg-key-id>
```

Before normal implementation commits, confirm signing works locally:

```bash
git commit-tree -S HEAD^{tree} -p HEAD -m "chore(repo): verify signing setup"
```

If signing is not configured on a bootstrap machine, record that in the task
completion note instead of silently disabling the project policy.

## Branch Naming

Use short, lowercase branches:

```text
feat/<short-scope>
fix/<short-scope>
chore/<short-scope>
docs/<short-scope>
security/<short-scope>
```

Do not force-push `main`. If a feature branch needs correction, use
`git push --force-with-lease` only after checking that no one else has pushed to
the same branch.

## Commit Messages

Use Conventional Commits:

```text
<type>(<scope>): <short imperative summary>
```

Allowed types are `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `build`,
`ci`, `chore`, `style`, `security`, and `revert`.

Recommended scopes include `openspec`, `repo`, `desktop`, `api`, `web`, `db`,
`ui`, `security`, `performance`, `optimizer`, `windows`, `nvidia`, `pubg`,
`benchmark`, and `release`.

Dry-run the commit message before committing:

```bash
echo "chore(repo): enforce signed conventional workflow" | npx --yes @commitlint/cli@19.8.1 --config .github/commitlint.config.cjs
```

## Local Commit Safety

Before every task commit:

```bash
git status --short
git diff --check
git diff --cached --check
```

Only stage files in the task write scope. Never commit `.env` files, private
keys, updater signing keys, Neon credentials, certificates, build artifacts, or
local database files.
