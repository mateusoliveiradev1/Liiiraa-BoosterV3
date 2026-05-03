export const supportedOptimizerLocales = ["en-US", "pt-BR", "es-ES"] as const;

export type OptimizerLocale = (typeof supportedOptimizerLocales)[number];

export const defaultOptimizerLocale: OptimizerLocale = "en-US";

export const optimizerLocaleFallbackOrder: readonly OptimizerLocale[] = [defaultOptimizerLocale];
export const optimizerLocaleStorageKey = "liiiraa.optimizer.locale";

const enUS = {
  meta: {
    name: "English (United States)",
    status: "default"
  },
  brand: {
    appName: "Liiiraa Booster",
    commandCenterAria: "Liiiraa Booster command center",
    signedBy: "Signed by Liiiraa"
  },
  shell: {
    sidebarAria: "Primary",
    navigationAria: "Desktop sections",
    statusStripAria: "Runtime status",
    routeMissing: "No desktop optimization route is configured."
  },
  commandHeader: {
    aria: "Route command header",
    controlsAria: "Route controls",
    nextAction: "Next action",
    secondaryControlsAria: "Secondary route controls",
    trustAria: "Trust and update context"
  },
  routes: {
    dashboard: {
      label: "Dashboard"
    },
    scan: {
      label: "Smart Scan"
    },
    optimize: {
      label: "Smart Boost"
    },
    power: {
      label: "Power"
    },
    nvidia: {
      label: "GPU Control"
    },
    pubg: {
      label: "Game Mode"
    },
    benchmarks: {
      label: "Performance"
    },
    rollback: {
      label: "Recovery"
    },
    settings: {
      label: "Settings"
    }
  },
  navigation: {
    titleWithGroup: "{group}: {label}",
    dashboard: {
      summary: "System score, trust, and next action"
    },
    scan: {
      summary: "Safe PC checkup"
    },
    optimize: {
      summary: "Safe Boost and review lanes"
    },
    power: {
      summary: "Scoped Windows power plans"
    },
    nvidia: {
      summary: "Driver and profile policy"
    },
    pubg: {
      group: "Games",
      summary: "PUBG profile and anti-cheat boundary"
    },
    benchmarks: {
      summary: "Live metrics and benchmark proof"
    },
    rollback: {
      summary: "Restore points and session timeline"
    },
    settings: {
      summary: "Privacy, updates, and trust"
    }
  },
  statusStrip: {
    scan: {
      label: "Scan",
      value: "{percent}%"
    },
    agent: {
      label: "Agent",
      valueReady: "Ready",
      detail: "No privileged changes are pending."
    },
    backups: {
      label: "Backups",
      value: "{count} sessions"
    },
    updater: {
      label: "Updater",
      detail: "{channel} channel, {transport}"
    }
  },
  glossary: {
    scan: "Scan",
    apply: "Apply",
    rollback: "Rollback",
    benchmark: "Benchmark",
    risk: "Risk",
    reboot: "Reboot",
    confidence: "Confidence",
    source: "Source",
    safe: "Safe",
    competitive: "Competitive",
    lab: "Lab",
    blocked: "Blocked"
  },
  labels: {
    available: "Available",
    change: "Change",
    impact: "Impact",
    item: "Item",
    before: "Before",
    after: "After",
    trust: "Trust",
    ready: "Ready",
    required: "Required",
    noMutation: "Not applied",
    none: "None"
  },
  risk: {
    aria: "{risk}: {label}",
    ariaWithDetail: "{risk}: {label}. {detail}",
    low: "Low risk",
    medium: "Moderate risk",
    high: "High risk",
    critical: "Critical risk",
    lab: "Lab only",
    lowShort: "Low",
    mediumShort: "Medium",
    highShort: "High",
    criticalShort: "Critical"
  },
  modes: {
    optimizationMode: "Boost mode",
    safeDescription: "Low-risk reversible changes only.",
    competitiveDescription: "Performance tradeoffs with explicit review.",
    labDescription: "Experimental changes behind per-category opt-in.",
    blockedDescription: "Educational items that cannot be applied."
  },
  actions: {
    startScan: "Start Smart Scan",
    cancelScan: "Cancel Smart Scan",
    generatePlan: "Open Smart Boost",
    continueScan: "Continue Smart Scan",
    retryScan: "Retry Smart Scan",
    reviewPlan: "Review plan",
    openRollback: "Open rollback",
    applySafeOnly: "Apply Safe Boost",
    applySafePlan: "Apply Safe Boost",
    includeCompetitive: "Review Competitive",
    inspectLab: "Inspect Lab",
    inspectLabItems: "Inspect lab items",
    exportPlan: "Export Boost Plan",
    cancel: "Cancel",
    stageBalanced: "Stage balanced",
    reviewCompetitive: "Review competitive",
    backupProfiles: "Back up profiles",
    stagePubgProfile: "Stage PUBG profile",
    openBenchmark: "Open benchmark",
    snapshotConfig: "Snapshot config",
    startDxBenchmark: "Start DX benchmark",
    openNvidiaProfile: "Open NVIDIA profile",
    captureBefore: "Capture before",
    compareAfter: "Compare after",
    exportReport: "Export report",
    restoreAll: "Restore all",
    restoreGpuProfiles: "Restore GPU profiles",
    exportAudit: "Export audit",
    checkUpdates: "Check updates",
    exportLocalData: "Export local data",
    openDataFolder: "Open data folder",
    previous: "Previous",
    next: "Next"
  },
  tooltips: {
    startScan: "Check your PC safely before Smart Boost opens.",
    cancelScan: "Stop Smart Scan. No changes are applied.",
    applySafeOnly: "Apply the reversible Safe Boost changes.",
    applySafePlan: "Apply the reversible Safe Boost changes.",
    includeCompetitive: "Include performance tradeoffs after explicit review.",
    inspectLab: "Open Lab-only recommendations without applying them.",
    exportPlan: "Export the Smart Boost plan.",
    cancel: "Cancel the current optimization review.",
    stageBalanced: "Stage the reversible balanced power plan.",
    reviewCompetitive: "Review competitive power tradeoffs before applying.",
    backupProfiles: "Back up NVIDIA profiles before staging changes.",
    stagePubgProfile: "Stage the PUBG profile after profile backup.",
    openBenchmark: "Open benchmark proof before applying GPU changes.",
    snapshotConfig: "Snapshot PUBG configuration before recommendations.",
    startDxBenchmark: "Start the DirectX benchmark comparison.",
    openNvidiaProfile: "Open the linked NVIDIA profile workflow.",
    captureBefore: "Capture the before benchmark run.",
    compareAfter: "Compare the after benchmark run.",
    exportReport: "Export benchmark proof and metadata.",
    rollbackSession: "Restore the previous optimization session.",
    restoreSelectedSession: "Restore selected session",
    restoreNvidiaProfileBackup: "Restore NVIDIA profile backup",
    exportRollbackAudit: "Export rollback audit",
    restoreAllChangesFromSession: "Restore all changes from {session}",
    restoreGpuProfilesFromSession: "Restore GPU profiles from {session}",
    showPreviousFrameSamples: "Show previous frame samples",
    showNextFrameSamples: "Show next frame samples"
  },
  primitives: {
    benchmarkProofAria: "Benchmark proof chart",
    storyAria: "Liiiraa primitive story render",
    systemStatus: "System status",
    tweakLedgerAria: "Boost change details",
    planActions: "Plan actions",
    metric: {
      measuring: "Measuring",
      delta: "Delta"
    }
  },
  workflow: {
    actions: {
      optimizationPlanAria: "Optimization plan actions",
      rollbackAria: "Rollback actions",
      sessionRollbackAria: "{session} rollback actions"
    },
    plan: {
      gatePolicyAria: "Smart Boost safety rules",
      tweaksAria: "Boost changes",
      riskLabel: "Risk label",
      consent: "Consent",
      defaultApply: "Ready to apply",
      noApplyControl: "Blocked from apply",
      reviewRequired: "Review required",
      applyControlEnabled: "Safe Boost is ready",
      noApplyControlRendered: "Blocked items stay informational",
      reviewOnlyUntilConsent: "Review required before apply",
      noExtraConsent: "No extra consent required",
      competitiveConsent: "Explicit performance tradeoff consent required",
      labConsent: "Advanced opt-in and benchmark framing required",
      deniedByPolicy: "Denied by safety policy",
      rebootMarked: "{count} marked",
      noRebootQueued: "No restart note is queued.",
      bucketChangeSummaries: "{count} changes in this lane.",
      blockedRollbackDetail: "Blocked items are never applied.",
      writeRollbackDetail: "A restore point is prepared before apply."
    },
    rollback: {
      noReboot: "No reboot",
      rebootRequired: "Reboot required",
      valuesAria: "Rollback values"
    }
  }
} as const;

type StringLeafPaths<T> = {
  [Key in keyof T & string]: T[Key] extends string
    ? Key
    : T[Key] extends Record<string, unknown>
      ? `${Key}.${StringLeafPaths<T[Key]>}`
      : never;
}[keyof T & string];

type WidenCatalog<T> = {
  readonly [Key in keyof T]: T[Key] extends string ? string : WidenCatalog<T[Key]>;
};

type DeepPartial<T> = {
  readonly [Key in keyof T]?: T[Key] extends string ? string : DeepPartial<T[Key]>;
};

export type OptimizerLocaleCatalog = WidenCatalog<typeof enUS>;
export type PartialOptimizerLocaleCatalog = DeepPartial<OptimizerLocaleCatalog>;
export type OptimizerLocaleKey = StringLeafPaths<typeof enUS>;
export type TranslationParams = Record<string, string | number | boolean | null | undefined>;

export interface MissingOptimizerLocaleKeySignal {
  key: string;
  locale: OptimizerLocale;
  fallbackLocale: OptimizerLocale | null;
  fallbackUsed: boolean;
}

export interface OptimizerTranslateOptions {
  catalogs?: Partial<Record<OptimizerLocale, PartialOptimizerLocaleCatalog>>;
  fallbackOrder?: readonly OptimizerLocale[];
  locale?: OptimizerLocale;
  signalMissingKeys?: boolean;
}

export const optimizerGlossaryKeys = {
  scan: "glossary.scan",
  apply: "glossary.apply",
  rollback: "glossary.rollback",
  benchmark: "glossary.benchmark",
  risk: "glossary.risk",
  reboot: "glossary.reboot",
  confidence: "glossary.confidence",
  source: "glossary.source",
  safe: "glossary.safe",
  competitive: "glossary.competitive",
  lab: "glossary.lab",
  blocked: "glossary.blocked"
} as const satisfies Record<string, OptimizerLocaleKey>;

export const optimizerLocaleCatalogs: Record<OptimizerLocale, PartialOptimizerLocaleCatalog> = {
  "en-US": enUS,
  "pt-BR": {
    meta: {
      name: "Portugues (Brasil)",
      status: "partial; falls back to en-US"
    },
    brand: {
      appName: "Liiiraa Booster",
      commandCenterAria: "Central de comando do Liiiraa Booster",
      signedBy: "Assinado pela Liiiraa"
    },
    shell: {
      sidebarAria: "Primario",
      navigationAria: "Secoes da area de trabalho",
      statusStripAria: "Status em tempo real",
      routeMissing: "Nenhuma rota de otimizacao da area de trabalho foi configurada."
    },
    commandHeader: {
      aria: "Cabecalho de comando da rota",
      controlsAria: "Controles da rota",
      nextAction: "Proxima acao",
      secondaryControlsAria: "Controles secundarios da rota",
      trustAria: "Contexto de confianca e atualizacao"
    },
    routes: {
      dashboard: {
        label: "Painel"
      },
      scan: {
        label: "Smart Scan"
      },
      optimize: {
        label: "Smart Boost"
      },
      power: {
        label: "Energia"
      },
      nvidia: {
        label: "Controle GPU"
      },
      pubg: {
        label: "Modo Jogo"
      },
      benchmarks: {
        label: "Performance"
      },
      rollback: {
        label: "Recuperacao"
      },
      settings: {
        label: "Ajustes"
      }
    },
    navigation: {
      titleWithGroup: "{group}: {label}",
      dashboard: {
        summary: "Score, confianca e proxima acao"
      },
      scan: {
        summary: "Checkup seguro do PC"
      },
      optimize: {
        summary: "Boost seguro e faixas de revisao"
      },
      power: {
        summary: "Planos de energia Windows com escopo"
      },
      nvidia: {
        summary: "Politica de driver e perfil"
      },
      pubg: {
        group: "Jogos",
        summary: "Perfil PUBG e limite anti-cheat"
      },
      benchmarks: {
        summary: "Metricas ao vivo e benchmark"
      },
      rollback: {
        summary: "Pontos de restauracao e sessoes"
      },
      settings: {
        summary: "Privacidade, updates e confianca"
      }
    },
    statusStrip: {
      scan: {
        label: "Scan",
        value: "{percent}%"
      },
      agent: {
        label: "Agente",
        valueReady: "Pronto",
        detail: "Nenhuma alteracao privilegiada esta pendente."
      },
      backups: {
        label: "Backups",
        value: "{count} sessoes"
      },
      updater: {
        label: "Atualizador",
        detail: "Canal {channel}, {transport}"
      }
    },
    glossary: {
      scan: "Scan",
      apply: "Aplicar",
      rollback: "Reversao",
      benchmark: "Benchmark",
      risk: "Risco",
      reboot: "Reinicio",
      confidence: "Confianca",
      source: "Fonte",
      safe: "Seguro",
      competitive: "Competitivo",
      lab: "Laboratorio",
      blocked: "Bloqueado"
    },
    labels: {
      available: "Disponivel",
      change: "Alteracao",
      impact: "Impacto",
      item: "Item",
      before: "Antes",
      after: "Depois",
      trust: "Confianca",
      ready: "Pronto",
      required: "Obrigatorio",
      noMutation: "Nao aplicado",
      none: "Nenhum"
    },
    risk: {
      aria: "{risk}: {label}",
      ariaWithDetail: "{risk}: {label}. {detail}",
      low: "Baixo risco",
      medium: "Risco moderado",
      high: "Alto risco",
      critical: "Risco critico",
      lab: "Somente laboratorio",
      lowShort: "Baixo",
      mediumShort: "Medio",
      highShort: "Alto",
      criticalShort: "Critico"
    },
    modes: {
      optimizationMode: "Modo do boost",
      safeDescription: "Somente alteracoes reversiveis de baixo risco.",
      competitiveDescription: "Trocas de desempenho com revisao explicita.",
      labDescription: "Alteracoes experimentais com opt-in por categoria.",
      blockedDescription: "Itens educativos que nao podem ser aplicados."
    },
    actions: {
      startScan: "Iniciar Smart Scan",
      cancelScan: "Cancelar Smart Scan",
      generatePlan: "Abrir Smart Boost",
      continueScan: "Continuar Smart Scan",
      retryScan: "Tentar Smart Scan novamente",
      reviewPlan: "Revisar plano",
      openRollback: "Abrir reversao",
      applySafeOnly: "Aplicar Safe Boost",
      applySafePlan: "Aplicar Safe Boost",
      includeCompetitive: "Revisar Competitivo",
      inspectLab: "Inspecionar Lab",
      inspectLabItems: "Inspecionar itens de laboratorio",
      exportPlan: "Exportar plano Smart Boost",
      cancel: "Cancelar",
      stageBalanced: "Preparar equilibrado",
      reviewCompetitive: "Revisar competitivo",
      backupProfiles: "Fazer backup dos perfis",
      stagePubgProfile: "Preparar perfil PUBG",
      openBenchmark: "Abrir benchmark",
      snapshotConfig: "Capturar configuracao",
      startDxBenchmark: "Iniciar benchmark DX",
      openNvidiaProfile: "Abrir perfil NVIDIA",
      captureBefore: "Capturar antes",
      compareAfter: "Comparar depois",
      exportReport: "Exportar relatorio",
      restoreAll: "Restaurar tudo",
      restoreGpuProfiles: "Restaurar perfis GPU",
      exportAudit: "Exportar auditoria",
      checkUpdates: "Verificar updates",
      exportLocalData: "Exportar dados locais",
      openDataFolder: "Abrir pasta de dados",
      previous: "Anterior",
      next: "Proximo"
    },
    tooltips: {
      startScan: "Verifica o PC com seguranca antes de abrir o Smart Boost.",
      cancelScan: "Para o Smart Scan. Nenhuma alteracao e aplicada.",
      applySafeOnly: "Aplica as alteracoes reversiveis do Safe Boost.",
      applySafePlan: "Aplica as alteracoes reversiveis do Safe Boost.",
      includeCompetitive: "Inclui trocas de desempenho depois da revisao explicita.",
      inspectLab: "Abre recomendacoes de laboratorio sem aplica-las.",
      exportPlan: "Exporta o plano Smart Boost.",
      cancel: "Cancela a revisao de otimizacao atual.",
      stageBalanced: "Prepara o plano equilibrado reversivel.",
      reviewCompetitive: "Revisa trocas competitivas de energia antes de aplicar.",
      backupProfiles: "Faz backup dos perfis NVIDIA antes de preparar alteracoes.",
      stagePubgProfile: "Prepara o perfil PUBG depois do backup de perfil.",
      openBenchmark: "Abre a prova de benchmark antes de aplicar alteracoes GPU.",
      snapshotConfig: "Captura a configuracao do PUBG antes das recomendacoes.",
      startDxBenchmark: "Inicia a comparacao de benchmark DirectX.",
      openNvidiaProfile: "Abre o fluxo de perfil NVIDIA vinculado.",
      captureBefore: "Captura a rodada de benchmark antes.",
      compareAfter: "Compara a rodada depois.",
      exportReport: "Exporta prova e metadados do benchmark.",
      rollbackSession: "Restaura a sessao de otimizacao anterior.",
      restoreSelectedSession: "Restaura a sessao selecionada",
      restoreNvidiaProfileBackup: "Restaura o backup de perfil NVIDIA",
      exportRollbackAudit: "Exporta a auditoria de reversao",
      restoreAllChangesFromSession: "Restaura todas as alteracoes de {session}",
      restoreGpuProfilesFromSession: "Restaura perfis GPU de {session}",
      showPreviousFrameSamples: "Mostra amostras de quadros anteriores",
      showNextFrameSamples: "Mostra proximas amostras de quadros"
    },
    primitives: {
      benchmarkProofAria: "Grafico de prova de benchmark",
      storyAria: "Renderizacao de primitives Liiiraa",
      systemStatus: "Status do sistema",
      tweakLedgerAria: "Detalhes das alteracoes do boost",
      planActions: "Acoes do plano",
      metric: {
        measuring: "Medindo",
        delta: "Delta"
      }
    },
    workflow: {
      actions: {
        optimizationPlanAria: "Acoes do plano de otimizacao",
        rollbackAria: "Acoes de reversao",
        sessionRollbackAria: "Acoes de reversao de {session}"
      },
      plan: {
        gatePolicyAria: "Regras de seguranca do Smart Boost",
        tweaksAria: "Alteracoes do boost",
        riskLabel: "Rotulo de risco",
        consent: "Consentimento",
        defaultApply: "Pronto para aplicar",
        noApplyControl: "Bloqueado para aplicar",
        reviewRequired: "Revisao obrigatoria",
        applyControlEnabled: "Safe Boost pronto",
        noApplyControlRendered: "Itens bloqueados ficam apenas informativos",
        reviewOnlyUntilConsent: "Revisao obrigatoria antes de aplicar",
        noExtraConsent: "Nenhum consentimento extra necessario",
        competitiveConsent: "Consentimento explicito de troca de desempenho necessario",
        labConsent: "Opt-in avancado e benchmark obrigatorios",
        deniedByPolicy: "Negado pela politica de seguranca",
        rebootMarked: "{count} marcados",
        noRebootQueued: "Nenhuma observacao de reinicio esta na fila.",
        bucketChangeSummaries: "{count} alteracoes nesta faixa.",
        blockedRollbackDetail: "Itens bloqueados nunca sao aplicados.",
        writeRollbackDetail: "Um ponto de restauracao e preparado antes de aplicar."
      },
      rollback: {
        noReboot: "Sem reinicio",
        rebootRequired: "Reinicio obrigatorio",
        valuesAria: "Valores de reversao"
      }
    }
  },
  "es-ES": {
    meta: {
      name: "Espanol (Espana)",
      status: "partial; falls back to en-US"
    },
    brand: {
      appName: "Liiiraa Booster",
      commandCenterAria: "Centro de comandos de Liiiraa Booster",
      signedBy: "Firmado por Liiiraa"
    },
    shell: {
      sidebarAria: "Primario",
      navigationAria: "Secciones de escritorio",
      statusStripAria: "Estado en tiempo real",
      routeMissing: "No hay una ruta de optimizacion de escritorio configurada."
    },
    commandHeader: {
      aria: "Cabecera de comandos de la ruta",
      controlsAria: "Controles de la ruta",
      nextAction: "Siguiente accion",
      secondaryControlsAria: "Controles secundarios de la ruta",
      trustAria: "Contexto de confianza y actualizacion"
    },
    routes: {
      dashboard: {
        label: "Panel"
      },
      scan: {
        label: "Smart Scan"
      },
      optimize: {
        label: "Smart Boost"
      },
      power: {
        label: "Energia"
      },
      nvidia: {
        label: "Control GPU"
      },
      pubg: {
        label: "Modo Juego"
      },
      benchmarks: {
        label: "Performance"
      },
      rollback: {
        label: "Recuperacion"
      },
      settings: {
        label: "Ajustes"
      }
    },
    navigation: {
      titleWithGroup: "{group}: {label}",
      dashboard: {
        summary: "Score, confianza y siguiente accion"
      },
      scan: {
        summary: "Chequeo seguro del PC"
      },
      optimize: {
        summary: "Boost seguro y carriles de revision"
      },
      power: {
        summary: "Planes de energia Windows con alcance"
      },
      nvidia: {
        summary: "Politica de driver y perfil"
      },
      pubg: {
        group: "Juegos",
        summary: "Perfil PUBG y limite anti-cheat"
      },
      benchmarks: {
        summary: "Metricas en vivo y benchmark"
      },
      rollback: {
        summary: "Puntos de restauracion y sesiones"
      },
      settings: {
        summary: "Privacidad, updates y confianza"
      }
    },
    statusStrip: {
      scan: {
        label: "Analisis",
        value: "{percent}%"
      },
      agent: {
        label: "Agente",
        valueReady: "Listo",
        detail: "No hay cambios privilegiados pendientes."
      },
      backups: {
        label: "Copias",
        value: "{count} sesiones"
      },
      updater: {
        label: "Actualizador",
        detail: "Canal {channel}, {transport}"
      }
    },
    glossary: {
      scan: "Analisis",
      apply: "Aplicar",
      rollback: "Reversion",
      benchmark: "Benchmark",
      risk: "Riesgo",
      reboot: "Reinicio",
      confidence: "Confianza",
      source: "Fuente",
      safe: "Seguro",
      competitive: "Competitivo",
      lab: "Laboratorio",
      blocked: "Bloqueado"
    },
    labels: {
      available: "Disponible",
      change: "Cambio",
      impact: "Impacto",
      item: "Elemento",
      before: "Antes",
      after: "Despues",
      trust: "Confianza",
      ready: "Listo",
      required: "Obligatorio",
      noMutation: "No aplicado",
      none: "Ninguno"
    },
    risk: {
      aria: "{risk}: {label}",
      ariaWithDetail: "{risk}: {label}. {detail}",
      low: "Riesgo bajo",
      medium: "Riesgo moderado",
      high: "Riesgo alto",
      critical: "Riesgo critico",
      lab: "Solo laboratorio",
      lowShort: "Bajo",
      mediumShort: "Medio",
      highShort: "Alto",
      criticalShort: "Critico"
    },
    modes: {
      optimizationMode: "Modo del boost",
      safeDescription: "Solo cambios reversibles de bajo riesgo.",
      competitiveDescription: "Intercambios de rendimiento con revision explicita.",
      labDescription: "Cambios experimentales con opt-in por categoria.",
      blockedDescription: "Elementos educativos que no se pueden aplicar."
    },
    actions: {
      startScan: "Iniciar Smart Scan",
      cancelScan: "Cancelar Smart Scan",
      generatePlan: "Abrir Smart Boost",
      continueScan: "Continuar Smart Scan",
      retryScan: "Reintentar Smart Scan",
      reviewPlan: "Revisar plan",
      openRollback: "Abrir reversion",
      applySafeOnly: "Aplicar Safe Boost",
      applySafePlan: "Aplicar Safe Boost",
      includeCompetitive: "Revisar Competitivo",
      inspectLab: "Inspeccionar Lab",
      inspectLabItems: "Inspeccionar elementos de laboratorio",
      exportPlan: "Exportar plan Smart Boost",
      cancel: "Cancelar",
      stageBalanced: "Preparar equilibrado",
      reviewCompetitive: "Revisar competitivo",
      backupProfiles: "Respaldar perfiles",
      stagePubgProfile: "Preparar perfil PUBG",
      openBenchmark: "Abrir benchmark",
      snapshotConfig: "Capturar configuracion",
      startDxBenchmark: "Iniciar benchmark DX",
      openNvidiaProfile: "Abrir perfil NVIDIA",
      captureBefore: "Capturar antes",
      compareAfter: "Comparar despues",
      exportReport: "Exportar informe",
      restoreAll: "Restaurar todo",
      restoreGpuProfiles: "Restaurar perfiles GPU",
      exportAudit: "Exportar auditoria",
      checkUpdates: "Buscar updates",
      exportLocalData: "Exportar datos locales",
      openDataFolder: "Abrir carpeta de datos",
      previous: "Anterior",
      next: "Siguiente"
    },
    tooltips: {
      startScan: "Comprueba el PC con seguridad antes de abrir Smart Boost.",
      cancelScan: "Detiene Smart Scan. No se aplica ningun cambio.",
      applySafeOnly: "Aplica los cambios reversibles de Safe Boost.",
      applySafePlan: "Aplica los cambios reversibles de Safe Boost.",
      includeCompetitive: "Incluye intercambios de rendimiento despues de la revision explicita.",
      inspectLab: "Abre recomendaciones de laboratorio sin aplicarlas.",
      exportPlan: "Exporta el plan Smart Boost.",
      cancel: "Cancela la revision de optimizacion actual.",
      stageBalanced: "Prepara el plan equilibrado reversible.",
      reviewCompetitive: "Revisa intercambios competitivos de energia antes de aplicar.",
      backupProfiles: "Respaldar perfiles NVIDIA antes de preparar cambios.",
      stagePubgProfile: "Prepara el perfil PUBG despues del respaldo de perfil.",
      openBenchmark: "Abre la prueba de benchmark antes de aplicar cambios GPU.",
      snapshotConfig: "Captura la configuracion de PUBG antes de las recomendaciones.",
      startDxBenchmark: "Inicia la comparacion de benchmark DirectX.",
      openNvidiaProfile: "Abre el flujo de perfil NVIDIA vinculado.",
      captureBefore: "Captura la corrida de benchmark antes.",
      compareAfter: "Compara la corrida despues.",
      exportReport: "Exporta prueba y metadatos del benchmark.",
      rollbackSession: "Restaura la sesion de optimizacion anterior.",
      restoreSelectedSession: "Restaura la sesion seleccionada",
      restoreNvidiaProfileBackup: "Restaura el respaldo de perfil NVIDIA",
      exportRollbackAudit: "Exporta la auditoria de reversion",
      restoreAllChangesFromSession: "Restaura todos los cambios de {session}",
      restoreGpuProfilesFromSession: "Restaura perfiles GPU de {session}",
      showPreviousFrameSamples: "Muestra muestras de cuadros anteriores",
      showNextFrameSamples: "Muestra siguientes muestras de cuadros"
    },
    primitives: {
      benchmarkProofAria: "Grafico de prueba de benchmark",
      storyAria: "Renderizado de primitives Liiiraa",
      systemStatus: "Estado del sistema",
      tweakLedgerAria: "Detalles de cambios del boost",
      planActions: "Acciones del plan",
      metric: {
        measuring: "Midiendo",
        delta: "Delta"
      }
    },
    workflow: {
      actions: {
        optimizationPlanAria: "Acciones del plan de optimizacion",
        rollbackAria: "Acciones de reversion",
        sessionRollbackAria: "Acciones de reversion de {session}"
      },
      plan: {
        gatePolicyAria: "Reglas de seguridad de Smart Boost",
        tweaksAria: "Cambios del boost",
        riskLabel: "Etiqueta de riesgo",
        consent: "Consentimiento",
        defaultApply: "Listo para aplicar",
        noApplyControl: "Bloqueado para aplicar",
        reviewRequired: "Revision obligatoria",
        applyControlEnabled: "Safe Boost listo",
        noApplyControlRendered: "Los elementos bloqueados quedan solo informativos",
        reviewOnlyUntilConsent: "Revision requerida antes de aplicar",
        noExtraConsent: "No requiere consentimiento adicional",
        competitiveConsent: "Consentimiento explicito de intercambio de rendimiento requerido",
        labConsent: "Opt-in avanzado y benchmark requeridos",
        deniedByPolicy: "Denegado por la politica de seguridad",
        rebootMarked: "{count} marcados",
        noRebootQueued: "No hay nota de reinicio en cola.",
        bucketChangeSummaries: "{count} cambios en este carril.",
        blockedRollbackDetail: "Los elementos bloqueados nunca se aplican.",
        writeRollbackDetail: "Se prepara un punto de restauracion antes de aplicar."
      },
      rollback: {
        noReboot: "Sin reinicio",
        rebootRequired: "Reinicio obligatorio",
        valuesAria: "Valores de reversion"
      }
    }
  }
};

const missingOptimizerLocaleKeys = new Map<string, MissingOptimizerLocaleKeySignal>();

export function isOptimizerLocale(value: string): value is OptimizerLocale {
  return supportedOptimizerLocales.includes(value as OptimizerLocale);
}

export function normalizeOptimizerLocale(value: string | null | undefined): OptimizerLocale {
  return value && isOptimizerLocale(value) ? value : defaultOptimizerLocale;
}

let activeOptimizerLocale: OptimizerLocale = getInitialOptimizerLocale();

export function getActiveOptimizerLocale(): OptimizerLocale {
  return activeOptimizerLocale;
}

export function setActiveOptimizerLocale(value: string | null | undefined): OptimizerLocale {
  activeOptimizerLocale = normalizeOptimizerLocale(value);
  persistOptimizerLocale(activeOptimizerLocale);

  return activeOptimizerLocale;
}

export function createOptimizerTranslator(
  locale: OptimizerLocale = defaultOptimizerLocale,
  options: Omit<OptimizerTranslateOptions, "locale"> = {}
) {
  return (key: OptimizerLocaleKey, params?: TranslationParams) =>
    translateOptimizerCopy(key, params, { ...options, locale });
}

export function translateOptimizerCopy(
  key: OptimizerLocaleKey,
  params: TranslationParams = {},
  options: OptimizerTranslateOptions = {}
): string {
  const locale = options.locale ?? defaultOptimizerLocale;
  const activeCatalog = options.catalogs?.[locale] ?? optimizerLocaleCatalogs[locale];
  const activeValue = readCatalogValue(activeCatalog, key);

  if (activeValue !== undefined) {
    return interpolate(activeValue, params);
  }

  const fallbackOrder = options.fallbackOrder ?? optimizerLocaleFallbackOrder;

  for (const fallbackLocale of fallbackOrder) {
    const fallbackCatalog = options.catalogs?.[fallbackLocale] ?? optimizerLocaleCatalogs[fallbackLocale];
    const fallbackValue = readCatalogValue(fallbackCatalog, key);

    if (fallbackValue !== undefined) {
      recordMissingLocaleKey({
        fallbackLocale,
        fallbackUsed: true,
        key,
        locale
      }, options.signalMissingKeys);

      return interpolate(fallbackValue, params);
    }
  }

  recordMissingLocaleKey({
    fallbackLocale: null,
    fallbackUsed: false,
    key,
    locale
  }, options.signalMissingKeys);

  return key;
}

export function tOptimizer(
  key: OptimizerLocaleKey,
  params?: TranslationParams,
  options: Omit<OptimizerTranslateOptions, "locale"> = {}
): string {
  return translateOptimizerCopy(key, params, { ...options, locale: activeOptimizerLocale });
}

export function getMissingOptimizerLocaleKeys(): MissingOptimizerLocaleKeySignal[] {
  return Array.from(missingOptimizerLocaleKeys.values());
}

export function clearMissingOptimizerLocaleKeys() {
  missingOptimizerLocaleKeys.clear();
}

export function assertNoMissingOptimizerLocaleKeys() {
  const missing = getMissingOptimizerLocaleKeys();

  if (missing.length > 0) {
    throw new Error(
      `Missing optimizer locale keys:\n${missing
        .map((item) => `${item.locale}:${item.key} -> ${item.fallbackLocale ?? "none"}`)
        .join("\n")}`
    );
  }
}

function getInitialOptimizerLocale(): OptimizerLocale {
  const urlLocale = readLocaleFromUrl();

  if (urlLocale) {
    persistOptimizerLocale(urlLocale);

    return urlLocale;
  }

  return readLocaleFromStorage() ?? defaultOptimizerLocale;
}

function readLocaleFromUrl(): OptimizerLocale | null {
  if (typeof globalThis.location === "undefined") {
    return null;
  }

  try {
    const locale = new URLSearchParams(globalThis.location.search).get("locale");

    return locale && isOptimizerLocale(locale) ? locale : null;
  } catch {
    return null;
  }
}

function readLocaleFromStorage(): OptimizerLocale | null {
  if (typeof globalThis.localStorage === "undefined") {
    return null;
  }

  try {
    const locale = globalThis.localStorage.getItem(optimizerLocaleStorageKey);

    return locale && isOptimizerLocale(locale) ? locale : null;
  } catch {
    return null;
  }
}

function persistOptimizerLocale(locale: OptimizerLocale) {
  if (typeof globalThis.localStorage === "undefined") {
    return;
  }

  try {
    globalThis.localStorage.setItem(optimizerLocaleStorageKey, locale);
  } catch {
    // Storage can be blocked in embedded shells; query-string locale still works for that session.
  }
}

function readCatalogValue(catalog: PartialOptimizerLocaleCatalog | undefined, key: string): string | undefined {
  let current: unknown = catalog;

  for (const segment of key.split(".")) {
    if (current == null || typeof current !== "object" || !(segment in current)) {
      return undefined;
    }

    current = (current as Record<string, unknown>)[segment];
  }

  return typeof current === "string" ? current : undefined;
}

function interpolate(template: string, params: TranslationParams) {
  return template.replace(/\{([A-Za-z0-9_]+)\}/g, (match, name: string) => {
    const value = params[name];

    return value == null ? match : String(value);
  });
}

function recordMissingLocaleKey(signal: MissingOptimizerLocaleKeySignal, forceSignal = false) {
  const signalId = `${signal.locale}:${signal.key}`;

  missingOptimizerLocaleKeys.set(signalId, signal);

  if (forceSignal || shouldSignalMissingLocaleKeys()) {
    console.warn(
      `[liiiraa-locale] Missing ${signal.locale} key "${signal.key}", fallback: ${
        signal.fallbackLocale ?? "none"
      }.`
    );
  }
}

function shouldSignalMissingLocaleKeys() {
  const env = (import.meta as ImportMeta & { env?: { DEV?: boolean; MODE?: string } }).env;

  return Boolean(env?.DEV || env?.MODE === "test");
}
