import tokens from "../tokens/liiiraa.tokens.json";

export const liiiraaTokens = tokens;

export type LiiiraaTokens = typeof tokens;
export type LiiiraaColorTokenGroup = keyof LiiiraaTokens["colors"];
export type LiiiraaTypographyTokenGroup = keyof LiiiraaTokens["typography"];
export type LiiiraaComponentTokenGroup = keyof LiiiraaTokens["components"];
