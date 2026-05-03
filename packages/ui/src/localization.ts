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
      summary: "Read-only system inventory"
    },
    optimize: {
      summary: "One-click safe tweaks"
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
      detail: "No privileged writes are pending."
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
    noMutation: "No mutation",
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
    optimizationMode: "Optimization mode",
    safeDescription: "Low-risk reversible changes only.",
    competitiveDescription: "Performance tradeoffs with explicit review.",
    labDescription: "Experimental changes behind per-category opt-in.",
    blockedDescription: "Educational items that cannot be applied."
  },
  actions: {
    startScan: "Start scan",
    cancelScan: "Cancel scan",
    generatePlan: "Generate plan",
    continueScan: "Continue scan",
    retryScan: "Retry scan",
    reviewPlan: "Review plan",
    openRollback: "Open rollback",
    applySafeOnly: "Apply safe only",
    applySafePlan: "Apply safe plan",
    includeCompetitive: "Include competitive",
    inspectLab: "Inspect lab",
    inspectLabItems: "Inspect lab items",
    exportPlan: "Export plan",
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
    startScan: "Start a read-only scan before any plan can be generated.",
    cancelScan: "Stop the current scan without applying changes.",
    applySafeOnly: "Apply only safe reversible changes.",
    applySafePlan: "Apply only safe reversible changes.",
    includeCompetitive: "Include performance tradeoffs after explicit review.",
    inspectLab: "Open Lab-only recommendations without applying them.",
    exportPlan: "Export the visible optimization plan.",
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
    tweakLedgerAria: "Audited tweak ledger",
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
      gatePolicyAria: "Plan group gate policy",
      tweaksAria: "Plan tweaks",
      riskLabel: "Risk label",
      consent: "Consent",
      defaultApply: "Default apply",
      noApplyControl: "No apply control",
      reviewRequired: "Review required",
      applyControlEnabled: "Apply control enabled for safe defaults",
      noApplyControlRendered: "No apply control is rendered for blocked rows",
      reviewOnlyUntilConsent: "Review-only until explicit consent",
      noExtraConsent: "No extra consent required",
      competitiveConsent: "Explicit performance tradeoff consent required",
      labConsent: "Advanced opt-in and benchmark framing required",
      deniedByPolicy: "Denied by safety policy",
      rebootMarked: "{count} marked",
      noRebootQueued: "No reboot prompt is queued.",
      bucketChangeSummaries: "{count} change summaries visible in this bucket.",
      blockedRollbackDetail: "Blocked rows have no apply control.",
      writeRollbackDetail: "Every write row keeps a rollback value."
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
        summary: "Inventario do sistema somente leitura"
      },
      optimize: {
        summary: "Tweaks seguros em um clique"
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
        detail: "Nenhuma escrita privilegiada esta pendente."
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
      noMutation: "Sem mutacao",
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
      optimizationMode: "Modo de otimizacao",
      safeDescription: "Somente alteracoes reversiveis de baixo risco.",
      competitiveDescription: "Trocas de desempenho com revisao explicita.",
      labDescription: "Alteracoes experimentais com opt-in por categoria.",
      blockedDescription: "Itens educativos que nao podem ser aplicados."
    },
    actions: {
      startScan: "Iniciar scan",
      cancelScan: "Cancelar scan",
      generatePlan: "Gerar plano",
      continueScan: "Continuar scan",
      retryScan: "Tentar scan novamente",
      reviewPlan: "Revisar plano",
      openRollback: "Abrir reversao",
      applySafeOnly: "Aplicar somente seguro",
      applySafePlan: "Aplicar plano seguro",
      includeCompetitive: "Incluir competitivo",
      inspectLab: "Inspecionar laboratorio",
      inspectLabItems: "Inspecionar itens de laboratorio",
      exportPlan: "Exportar plano",
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
    workflow: {
      actions: {
        optimizationPlanAria: "Acoes do plano de otimizacao",
        rollbackAria: "Acoes de reversao",
        sessionRollbackAria: "Acoes de reversao de {session}"
      },
      plan: {
        gatePolicyAria: "Politica de portao do plano",
        tweaksAria: "Ajustes do plano",
        riskLabel: "Rotulo de risco",
        consent: "Consentimento",
        defaultApply: "Aplicacao padrao",
        noApplyControl: "Sem controle de aplicacao",
        reviewRequired: "Revisao obrigatoria",
        applyControlEnabled: "Controle de aplicacao habilitado para padroes seguros",
        noApplyControlRendered: "Nenhum controle de aplicacao e renderizado para linhas bloqueadas",
        reviewOnlyUntilConsent: "Somente revisao ate consentimento explicito",
        noExtraConsent: "Nenhum consentimento extra necessario",
        competitiveConsent: "Consentimento explicito de troca de desempenho necessario",
        labConsent: "Opt-in avancado e benchmark obrigatorios",
        deniedByPolicy: "Negado pela politica de seguranca",
        rebootMarked: "{count} marcados",
        noRebootQueued: "Nenhum reinicio esta na fila.",
        bucketChangeSummaries: "{count} resumos de alteracao visiveis neste grupo.",
        blockedRollbackDetail: "Linhas bloqueadas nao tem controle de aplicacao.",
        writeRollbackDetail: "Cada escrita mantem um valor de reversao."
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
        summary: "Inventario del sistema solo lectura"
      },
      optimize: {
        summary: "Ajustes seguros en un clic"
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
        detail: "No hay escrituras privilegiadas pendientes."
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
      noMutation: "Sin mutacion",
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
      optimizationMode: "Modo de optimizacion",
      safeDescription: "Solo cambios reversibles de bajo riesgo.",
      competitiveDescription: "Intercambios de rendimiento con revision explicita.",
      labDescription: "Cambios experimentales con opt-in por categoria.",
      blockedDescription: "Elementos educativos que no se pueden aplicar."
    },
    actions: {
      startScan: "Iniciar analisis",
      cancelScan: "Cancelar analisis",
      generatePlan: "Generar plan",
      continueScan: "Continuar analisis",
      retryScan: "Reintentar analisis",
      reviewPlan: "Revisar plan",
      openRollback: "Abrir reversion",
      applySafeOnly: "Aplicar solo seguro",
      applySafePlan: "Aplicar plan seguro",
      includeCompetitive: "Incluir competitivo",
      inspectLab: "Inspeccionar laboratorio",
      inspectLabItems: "Inspeccionar elementos de laboratorio",
      exportPlan: "Exportar plan",
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
    workflow: {
      actions: {
        optimizationPlanAria: "Acciones del plan de optimizacion",
        rollbackAria: "Acciones de reversion",
        sessionRollbackAria: "Acciones de reversion de {session}"
      },
      plan: {
        gatePolicyAria: "Politica de puerta del plan",
        tweaksAria: "Ajustes del plan",
        riskLabel: "Etiqueta de riesgo",
        consent: "Consentimiento",
        defaultApply: "Aplicacion predeterminada",
        noApplyControl: "Sin control de aplicacion",
        reviewRequired: "Revision obligatoria",
        applyControlEnabled: "Control de aplicacion habilitado para valores seguros",
        noApplyControlRendered: "No se muestra control de aplicacion para filas bloqueadas",
        reviewOnlyUntilConsent: "Solo revision hasta consentimiento explicito",
        noExtraConsent: "No requiere consentimiento adicional",
        competitiveConsent: "Consentimiento explicito de intercambio de rendimiento requerido",
        labConsent: "Opt-in avanzado y benchmark requeridos",
        deniedByPolicy: "Denegado por la politica de seguridad",
        rebootMarked: "{count} marcados",
        noRebootQueued: "No hay reinicio en cola.",
        bucketChangeSummaries: "{count} resumenes de cambio visibles en este grupo.",
        blockedRollbackDetail: "Las filas bloqueadas no tienen control de aplicacion.",
        writeRollbackDetail: "Cada escritura mantiene un valor de reversion."
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
