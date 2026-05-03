---
status: resolved
trigger: "--auto auditoria geral do projeto inteiro o app e um app de otimizaçao do windows e ele precisa ser perfeito"
created: 2026-05-03
updated: 2026-05-03
---

# Debug Session: Auto Auditoria Geral Projeto

## Symptoms

- expected_behavior: O app inteiro de otimizacao do Windows deve estar funcional, confiavel e polido.
- actual_behavior: Auditoria geral solicitada; nao ha um erro unico informado.
- error_messages: Nenhuma mensagem especifica informada no inicio.
- timeline: Validacao final/ampla solicitada em 2026-05-03.
- reproduction: Rodar verificacoes de codigo, contratos de tweaks, build/testes e validar UI/copy dos fluxos principais.

## Current Focus

- hypothesis: A auditoria encontrou uma regressao real em teste de workflow desatualizado, warnings Rust evitaveis, fallbacks de locale em tooltips/ARIA e um problema visual de label/detalhe colado.
- test: Rodar checks automatizados, testes visuais Playwright, cargo check/test, build e inspecao manual de screenshot do fluxo Smart Boost em pt-BR.
- expecting: Build, testes, contratos de tweaks, UI e copy principal passam sem erro reproduzivel.
- next_action: complete
- reasoning_checkpoint:
- tdd_checkpoint:

## Evidence

- timestamp: 2026-05-03T18:40:00-03:00
  observation: `pnpm check:js` falhou em `apps/desktop/tests/optimization-workflow.spec.mjs` porque o teste ainda esperava "Read-only system scan" enquanto a UI atual usa Smart Scan/Smart Boost.
- timestamp: 2026-05-03T18:41:00-03:00
  observation: `cargo check --workspace --all-targets` compilou, mas mostrou warnings de imports Rust usados apenas em testes e crates de teste sem uso explicito de dependencias.
- timestamp: 2026-05-03T18:41:00-03:00
  observation: Playwright exibiu warnings de locale para tooltips/primitives em pt-BR e es-ES.
- timestamp: 2026-05-03T18:47:00-03:00
  observation: Inspecao visual do screenshot pt-BR de Smart Boost revelou labels colados aos detalhes em `StatusRow`.
- timestamp: 2026-05-03T18:55:00-03:00
  observation: `pnpm check:js`, `pnpm build`, `cargo check --workspace --all-targets` e `cargo test --workspace` passaram.
- timestamp: 2026-05-03T18:59:00-03:00
  observation: `pnpm desktop:build` compilou o executavel release e gerou o instalador NSIS, mas terminou com erro porque existe chave publica Tauri sem `TAURI_SIGNING_PRIVATE_KEY` no ambiente local.

## Eliminated

- hypothesis: Contratos de tweaks permitem alteracoes perigosas por padrao.
  result: Eliminada por testes de catalogo/optimizer-core/windows-api cobrindo Safe, Competitive, Lab, Blocked, rollback, backup, Defender, anti-cheat, security tradeoffs e Windows Update.
- hypothesis: UI principal tem overflow, clipping ou tela vazia nas rotas desktop.
  result: Eliminada por Playwright em 1024x680, 1280x800 e 1440x900, com screenshots e checks de overflow/clipping/overlap.

## Resolution

- root_cause: Teste de workflow e algumas superficies de copy/layout ficaram atrasados em relacao ao redesign Smart Scan/Smart Boost e ao suporte parcial pt-BR/es-ES.
- fix: Atualizado o teste de workflow para a copy real, adicionadas traducoes de tooltips/primitives em pt-BR e es-ES, corrigido `StatusRow` para separar label e detalhe, e removido ruido Rust movendo imports/test deps para os lugares corretos.
- verification: `pnpm check:js`; `pnpm build`; `pnpm --filter @liiiraa/desktop test`; `cargo check --workspace --all-targets`; `cargo test --workspace`; `pnpm desktop:build` ate geracao de EXE/NSIS, bloqueado apenas na etapa local de assinatura por falta de `TAURI_SIGNING_PRIVATE_KEY`.
- files_changed: apps/desktop/tests/optimization-workflow.spec.mjs; apps/desktop/src/components/OptimizationWorkflow.tsx; packages/ui/src/localization.ts; crates/windows-api/src/defender.rs; crates/windows-api/src/network.rs; crates/windows-api/src/security_tradeoff.rs; crates/windows-api/src/storage.rs; crates/windows-api/tests/optimizer_integration.rs; crates/windows-api/tests/system_scan.rs.
