# Multi-Asset Farm Implementation Guide for Frontend

## Overview

The rewards simulator now supports **multi-asset farms** — farms that split their protocol deposits across multiple asset types (USDG, GLW, and/or SGCTL). This document explains how to integrate with the new API endpoint.

---

## New API Endpoint

```
POST /api/rewards-simulator-multi-asset
```

This is a **new endpoint** — the existing `/api/rewards-simulator` endpoint is unchanged and still works for single-asset farms.

---

## Key Differences from Single-Asset Farms

| Aspect | Single-Asset (Old) | Multi-Asset (New) |
|--------|-------------------|-------------------|
| Endpoint | `/api/rewards-simulator` | `/api/rewards-simulator-multi-asset` |
| Deposit field | `protocolDepositValue` + `assetId` | `totalProtocolDepositValue` + `assets[]` array |
| Impact | `netWeeklyImpactAssets` | Same, but distributed proportionally across assets |
| Output rewards | Single `assetEarned` per farm | `assets[]` array with earnings per asset type |

---

## Naming Convention Note

All API field names use **camelCase** for consistency. In particular:

- `quotedByGvePricePerAsset` (not `quotedByGVEPricePerAsset`)

This ensures uniformity across all fields in requests and responses.

---

## Input Format

### Farm Object Structure

```json
{
  "farmId": "my-farm-123",
  "regionId": 3,
  "netWeeklyImpactAssets": "1570000000000000000000",
  "firstWeek": 102,
  "weeksAlive": 52,
  "totalProtocolDepositValue": "250000000000",
  "assets": [
    {
      "assetId": "GLW",
      "assetsRequired": "250000000000000000000000",
      "assetsRequiredUSDC": "100000000000",
      "quotedByGvePricePerAsset": "400000",
      "decimals": 18
    },
    {
      "assetId": "USDG",
      "assetsRequiredUSDC": "75000000000",
      "assetsRequired": "75000000000",
      "quotedByGvePricePerAsset": "1000000",
      "decimals": 6
    },
    {
      "assetId": "SGCTL",
      "assetsRequiredUSDC": "75000000000",
      "assetsRequired": "75000000000",
      "quotedByGvePricePerAsset": "1000000",
      "decimals": 6
    }
  ],
  "rewardSplit": [
    {
      "walletAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
      "glowSplitPercent6Decimals": "1000000",
      "depositSplitPercent6Decimals": "1000000"
    }
  ]
}
```

---

## Output Format

### Farm Rewards (per week)

```json
{
  "102": {
    "farmRewards": [
      {
        "id": "my-farm-123",
        "farmId": "my-farm-123",
        "weekIndex": 102,
        "regionId": 3,
        "assets": [
          { "assetId": "GLW", "assetEarned": "1230000000000000000000" },
          { "assetId": "SGCTL", "assetEarned": "450000000" },
          { "assetId": "USDG", "assetEarned": "250000000" }
        ],
        "glowInflationReward": "900000000000000000000",
        "protocolDeposit": "250000000000",
        "expectedProduction": "1570000000000000000000"
      }
    ],
    "walletDistributions": [...],
    "regionData": {...},
    "warnings": []
  }
}
```

### Key Output Differences

| Field | Single-Asset | Multi-Asset |
|-------|--------------|-------------|
| Per-farm asset rewards | `assetEarned: "123..."` | `assets: [{ assetId, assetEarned }, ...]` |
| Wallet traces | One trace per farm | Multiple traces per farm (one per asset) |

---

## Complete Request Example

```bash
curl -X POST http://localhost:35025/api/rewards-simulator-multi-asset \
-H "Content-Type: application/json" \
-d '{
  "cgpLeftovers": {},
  "solarFarms": [
    {
      "farmId": "farm-1",
      "regionId": 3,
      "netWeeklyImpactAssets": "1000000000000000000000",
      "firstWeek": 100,
      "weeksAlive": 52,
      "totalProtocolDepositValue": "100000000000",
      "assets": [
        {
          "assetId": "GLW",
          "assetsRequired": "250000000000000000000000",
          "assetsRequiredUSDC": "50000000000",
          "quotedByGvePricePerAsset": "200000",
          "decimals": 18
        },
        {
          "assetId": "USDG",
          "assetsRequired": "50000000000",
          "assetsRequiredUSDC": "50000000000",
          "quotedByGvePricePerAsset": "1000000",
          "decimals": 6
        }
      ],
      "rewardSplit": [
        {
          "walletAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
          "glowSplitPercent6Decimals": "1000000",
          "depositSplitPercent6Decimals": "1000000"
        }
      ]
    }
  ],
  "gctlDistribution": {}
}'
```

---

## Reference Implementation

See `src/web/js/ma_api.js` for a working frontend implementation of the multi-asset API integration.

---

## Validation Rules Summary

1. Each farm needs at least one asset in `assets[]`
2. `assetId` must be `"USDG"`, `"GLW"`, or `"SGCTL"`
3. Sum of all `assetsRequiredUSDC` must equal `totalProtocolDepositValue`
4. `quotedByGvePricePerAsset` required for each asset
5. `rewardSplit` percentages must each sum to `1000000` (100%)
6. `weeksAlive` must be at least 2
