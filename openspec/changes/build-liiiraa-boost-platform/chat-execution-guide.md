# Chat Execution Guide

Use this guide when starting fresh chats for implementation. The cleanest approach is one task ID per chat.

Before editing, open `tasks.md`, locate the selected row, and treat that row as the task contract. Read every file in `Read First`, edit only `Write Scope`, run `Verify` or the closest valid check, commit with `Commit and Push`, push the active branch, and stop without starting the next task.

## Recommended: One Task Per Chat

Use this prompt:

```text
Use the OpenSpec change `build-liiiraa-boost-platform`.

Execute only task T000 from:
openspec/changes/build-liiiraa-boost-platform/tasks.md

Rules:
- Read the task row and all files listed in "Read First".
- Edit only the listed "Write Scope".
- Do not start the next task.
- Run the listed verification or the closest valid check.
- Commit with the suggested Conventional Commit message.
- Push the active branch.
- Final answer must include Verification, Commit, Push, and Files changed.
```

Then replace `T000` with the next task ID.

For Phase 0, open separate chats:
- Chat 1: `T000`
- Chat 2: `T001`
- Chat 3: `T002`
- Chat 4: `T003`
- Chat 5: `T004`
- Chat 6: `T005`
- Chat 7: `T006`

## Acceptable: One Phase Per Chat

Only use this when you want fewer chats and accept more context in one thread.

```text
Use the OpenSpec change `build-liiiraa-boost-platform`.

Execute Phase 0 only: T000 through T006 from:
openspec/changes/build-liiiraa-boost-platform/tasks.md

Rules:
- Complete tasks sequentially.
- Commit and push after each task ID, not only at the end.
- Stop after T006.
- Do not implement Phase 1.
- Final answer must list each task ID with Verification, Commit, Push, and Files changed.
```

## Best Default for This Project

Use one task per chat for implementation-heavy tasks. Use one phase per chat only for small documentation/governance phases.

Do not ask a new chat to "build the whole app". Ask for a specific task ID.

Every final implementation answer must include `Verification`, `Commit`, `Push`, and `Files changed`, matching the completion template in `tasks.md`.
