# Definition of Done

This file applies to every task ID in `tasks.md`. A task is not complete until every relevant item below is complete and the task's `Commit and Push` column has been satisfied or explicitly reported as blocked.

`tasks.md` is the executable task map and source of truth for handoff. If supporting OpenSpec docs conflict with the selected task row, the task row wins until the OpenSpec docs are intentionally updated by a governance task.

## Universal Done
- The task is implemented in the smallest coherent scope.
- The implementation matches the relevant spec file.
- The implementation does not cross unrelated module boundaries.
- The smallest meaningful verification passed.
- Risky behavior has negative tests.
- Secrets were not added.
- Generated files are intentional.
- The task was committed with a Conventional Commit message.
- The commit was signed when local signing is configured.
- The branch was pushed after the coherent micro-step when authentication is available.

## Required Commit Footer for Risky Work
For security-sensitive, privileged, updater, release, tweak, or data-migration work, the commit body or footer should include verification notes.

Example:
```text
security(agent): deny unknown elevated commands

Verification:
- cargo test -p windows-agent
- pnpm test --filter desktop

Risk:
- Blocks unknown command IDs before privilege boundary.
```

## Micro-Step Rule
One task may produce multiple commits if the task naturally splits into smaller implementation slices. Do not bundle unrelated work to reduce commit count.

Good:
```text
build(repo): initialize pnpm workspace
build(repo): add turborepo pipeline
test(repo): add workspace verification script
```

Bad:
```text
feat(app): add desktop api db nvidia pubg and landing page
```

## Push Rule
Push after each coherent green micro-step.

Allowed exception:
- local credentials are unavailable
- commit signing is not configured yet
- the user explicitly asks to pause before push
- the branch is intentionally local for a spike

When push is skipped, the final/status message must state why.

## AI Agent Done
Any AI agent working in this repo must end each implementation turn with:
- exact task ID executed
- changed files
- verification run
- commit hash if committed
- push status
- blockers or risks
