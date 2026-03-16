import { z } from "zod";

export const operationalModeSchema = z.enum([
  "research",
  "backtest",
  "paper",
  "live",
]);

export const directionSchema = z.enum(["long", "short", "neutral"]);
export const actionKindSchema = z.enum([
  "hold",
  "watch",
  "open_long",
  "open_short",
  "reduce",
  "close",
  "hedge",
]);

export const evidenceItemSchema = z.object({
  summary: z.string(),
  sourceId: z.string(),
  weight: z.number(),
});

export const modelOutputSchema = z.object({
  id: z.string().uuid(),
  modelName: z.string(),
  role: z.enum([
    "reasoning",
    "market_structure",
    "sentiment_news",
    "quant_predictive",
    "risk",
    "fusion",
  ]),
  thesis: z.string(),
  direction: directionSchema,
  timeHorizon: z.string(),
  confidence: z.number(),
  supportingEvidence: z.array(evidenceItemSchema),
  invalidationConditions: z.array(z.string()),
  riskNotes: z.array(z.string()),
  recommendedAction: actionKindSchema,
  recommendedSize: z.number(),
  abstain: z.boolean(),
  latencyMs: z.number(),
  costUsd: z.number(),
  sourceRefs: z.array(z.string()),
  generatedAt: z.string(),
});

export const dissentingViewSchema = z.object({
  modelName: z.string(),
  reason: z.string(),
  direction: directionSchema,
  confidence: z.number(),
});

export const fusedThesisSchema = z.object({
  id: z.string().uuid(),
  market: z.string(),
  regime: z.string(),
  thesis: z.string(),
  direction: directionSchema,
  timeHorizon: z.string(),
  confidence: z.number(),
  disagreementScore: z.number(),
  overconfidenceFlag: z.boolean(),
  recommendedAction: actionKindSchema,
  recommendedSize: z.number(),
  abstain: z.boolean(),
  supportingEvidence: z.array(evidenceItemSchema),
  dissentingViews: z.array(dissentingViewSchema),
  whyNot: z.array(z.string()),
  modelOutputs: z.array(modelOutputSchema),
  sourceRefs: z.array(z.string()),
  generatedAt: z.string(),
});

export const riskDecisionSchema = z.object({
  id: z.string().uuid(),
  thesisId: z.string().uuid(),
  approved: z.boolean(),
  status: z.enum([
    "approved",
    "rejected",
    "requires_human_approval",
    "staged",
  ]),
  reasons: z.array(z.string()),
  cappedSize: z.number(),
  requiredApprovals: z.array(z.string()),
  killSwitchArmed: z.boolean(),
  evaluatedAt: z.string(),
});

export const chatRequestSchema = z.object({
  actor: z.string(),
  mode: operationalModeSchema,
  market: z.string(),
  message: z.string(),
  watchlist: z.array(z.string()),
});

export const chatResponseSchema = z.object({
  narrative: z.string(),
  decisionSummary: z.string(),
  thesis: fusedThesisSchema,
  risk: riskDecisionSchema,
  citations: z.array(z.string()),
  machinePayload: z.record(z.string(), z.unknown()),
});

export const healthSnapshotSchema = z.object({
  service: z.string(),
  mode: operationalModeSchema,
  healthy: z.boolean(),
  details: z.record(z.string(), z.string()),
  checkedAt: z.string(),
});

export type OperationalMode = z.infer<typeof operationalModeSchema>;
export type Direction = z.infer<typeof directionSchema>;
export type ActionKind = z.infer<typeof actionKindSchema>;
export type EvidenceItem = z.infer<typeof evidenceItemSchema>;
export type ModelOutput = z.infer<typeof modelOutputSchema>;
export type FusedThesis = z.infer<typeof fusedThesisSchema>;
export type RiskDecision = z.infer<typeof riskDecisionSchema>;
export type ChatRequest = z.infer<typeof chatRequestSchema>;
export type ChatResponse = z.infer<typeof chatResponseSchema>;
export type HealthSnapshot = z.infer<typeof healthSnapshotSchema>;
