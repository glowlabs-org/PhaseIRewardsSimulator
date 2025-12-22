# `rewards-simulator` API Documentation

This document provides a detailed explanation of the HTTP API for the `rewards-simulator` service.

## Endpoints

The `rewards-simulator` exposes two primary API endpoints for running simulations:

*   `POST /api/rewards-simulator`
*   `POST /api/rewards-simulator-detailed`

---

## `POST /api/rewards-simulator`

### Description

This endpoint is the main entry point for running a rewards simulation. It accepts a JSON object describing a set of solar farms and other competition parameters. It returns a JSON object containing the calculated weekly rewards, structured for easy consumption by downstream systems. The output summarizes rewards on both a per-wallet and per-farm basis for each week of the simulation.

### Request Body Parameters

The JSON request body has the following top-level fields:

*   `cgpLeftovers` (optional): A map where keys are week numbers (as strings) and values are the amount of USDG to be added to that week's CGP competition. This is used for distributing early liquidity rewards. The amount is a string scaled by 1e6.
*   `solarFarms` (required): An array of `SolarFarm` objects. Each object describes a farm's parameters, such as its deposit value, impact, and reward distribution splits.
*   `gctlDistribution` (optional): A map where keys are region IDs (as strings) and values are the amount of GCTL tokens staked to that region, scaled by 1e18. If provided, this field determines how GLW inflation rewards are distributed among regions.
*   `outputFarms` (optional): An array of strings, where each string is a `farmId`. If this field is provided, the API response for each week will be filtered to only include the `farmRewards` and `warnings` fields, and the `farmRewards` array will only contain entries for the specified farm IDs. This is useful for clients who only need per-farm reward data without wallet-level details.

### Sample Input

```json
{
  "cgpLeftovers": {
    "97": "235000000"
  },
  "solarFarms": [
    {
      "farmId": "45-bb",
      "assetId": "USDG",
      "regionId": 1,
      "netWeeklyImpactAssets": "23000000000000000000",
      "protocolDepositValue": "10000000",
      "assetsRequired": "10000000",
      "firstWeek": 96,
      "weeksAlive": 100,
      "rewardSplit": [
        {
          "walletAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
          "glowSplitPercent6Decimals": "1000000",
          "depositSplitPercent6Decimals": "1000000"
        }
      ]
    },
    {
      "farmId": "90-fa",
      "assetId": "GLW",
      "regionId": 2,
      "netWeeklyImpactAssets": "45000000000000000000",
      "protocolDepositValue": "6000000",
      "assetsRequired": "12000000000000000000",
      "firstWeek": 96,
      "weeksAlive": 60,
      "rewardSplit": [
        {
          "walletAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
          "glowSplitPercent6Decimals": "1000000",
          "depositSplitPercent6Decimals": "1000000"
        }
      ]
    }
  ],
  "gctlDistribution": {
    "1": "135000000000000000000000",
    "2": "35000000000000000000000"
  },
  "outputFarms": [
    "45-bb"
  ]
}
```

### Example Code for Calling API

Here is an example of how to call the endpoint using `curl`.

```bash
curl -X POST http://localhost:35025/api/rewards-simulator \
-H "Content-Type: application/json" \
-d '{
  "cgpLeftovers": {
    "97": "235000000"
  },
  "solarFarms": [
    {
      "farmId": "45-bb",
      "assetId": "USDG",
      "regionId": 1,
      "netWeeklyImpactAssets": "23000000000000000000",
      "protocolDepositValue": "10000000",
      "assetsRequired": "10000000",
      "firstWeek": 96,
      "weeksAlive": 100,
      "rewardSplit": [
        {
          "walletAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
          "glowSplitPercent6Decimals": "1000000",
          "depositSplitPercent6Decimals": "1000000"
        }
      ]
    }
  ],
  "outputFarms": ["45-bb"]
}'
```

### Sample Response

The response is a JSON object mapping week numbers to their reward data.

```json
{
  "97": {
    "walletDistributions": [
      {
        "userAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
        "assetsEarned": {
           "USDG": "95000"
        },
        "glowInflationEarned": "498000000000000000000",
        "traces": [
          {
             "farmId": "45-bb",
             "asset": "USDG",
             "inflationRewardSplit6Decimals": "1000000",
             "depositRewardSplit6Decimals": "1000000",
             "amount": "95000",
             "regionId": 1,
             "glowInflationReward": "498000000000000000000"
          }
        ]
      }
    ],
    "farmRewards": [
      {
        "id": "45-bb",
        "asset": "USDG",
        "regionId": 1,
        "assetEarned": "95000",
        "glowInflationReward": "498000000000000000000",
        "protocolDeposit": "10000000",
        "expectedProduction": "23000000000000000000"
      }
    ],
    "regionData": {
      "1": {
        "USDG": {
          "protocolDepositSum": "10000000",
          "carbonCreditProductionSum": "23000000000000000000"
        }
      }
    },
    "warnings": [
      "this is an example warning"
    ]
  }
}
```

### Sample Response (with `outputFarms`)

When the `outputFarms` parameter is used, the response for each week is reduced to just the `farmRewards` and `warnings` arrays. The `farmRewards` array is filtered to only include the requested farms.

```json
{
  "97": {
    "farmRewards": [
      {
        "id": "45-bb",
        "asset": "USDG",
        "regionId": 1,
        "assetEarned": "95000",
        "glowInflationReward": "498000000000000000000",
        "protocolDeposit": "10000000",
        "expectedProduction": "23000000000000000000"
      }
    ],
    "warnings": [
      "this is an example warning"
    ]
  }
}
```

### Query Parameters

The endpoint supports the following query parameters:

*   `preloadGlowV1`: A string that, if set to `true`, will cause the simulator to load farm data from a predefined `v1-data.json` file and merge it with the input provided in the request body. This is useful for simulations that need to include historical context.
*   `week`: A string representing a week number. If this parameter is provided, the API will return only the rewards data for that specific week, rather than the full map of all weeks.

### Response Codes and Errors

*   **200 OK**: The simulation completed successfully without any internal consistency errors. The response body contains the full JSON output of weekly rewards. If the `week` parameter was used, the body contains the data for just that week.
*   **400 Bad Request**: The input JSON failed validation. This can be due to missing fields, incorrect data types, or values that are out of bounds (e.g., negative numbers where positive are expected). The response body will be a JSON object with an `error` key containing a descriptive message, for example: `{"error": "validation error: no farms provided"}`.
*   **404 Not Found**: This code is returned only when the `week` query parameter is used and the requested week does not exist in the simulation results. The response body will be a JSON object like: `{"error": "requested week not found: 12345"}`.
*   **422 Unprocessable Entity**: The simulation ran to completion, but one or more internal consistency checks failed during the computation. This indicates a potential issue with the algorithm's state. The response body will be a JSON object with two keys: `errors`, an array of strings detailing the consistency issues, and `output`, the full simulation output, which can be inspected for debugging purposes.
*   **500 Internal Server Error**: An unexpected error occurred on the server, such as a failure to read a required file like `v1-data.json`. The response body will contain a JSON object with an `error` key.

---

## `POST /api/rewards-simulator-detailed`

### Description

This endpoint provides a much more detailed, low-level view of the simulation's internal state. It is primarily intended for debugging, diagnostics, and advanced data visualization. The response includes the full data structures for competitions, weekly buckets, and the state of each farm within each bucket. An alias for this endpoint exists at `/ui/rewards-simulator-detailed`, which is used by the frontend visualizer.

### Request Body Parameters

The request body is identical to `/api/rewards-simulator`. However, the `outputFarms` field is ignored by this endpoint; the full detailed output is always returned.

### Sample Input

The input format is identical to the `/api/rewards-simulator` endpoint.

```json
{
  "cgpLeftovers": {},
  "solarFarms": [
    {
      "farmId": "45-bb",
      "assetId": "USDG",
      "regionId": 1,
      "netWeeklyImpactAssets": "23000000000000000000",
      "protocolDepositValue": "10000000",
      "assetsRequired": "10000000",
      "firstWeek": 96,
      "weeksAlive": 100,
      "rewardSplit": [
        {
          "walletAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
          "glowSplitPercent6Decimals": "1000000",
          "depositSplitPercent6Decimals": "1000000"
        }
      ]
    }
  ],
  "gctlDistribution": {
    "1": "135000000000000000000000"
  },
  "outputFarms": ["45-bb"]
}
```

### Example Code for Calling API

```bash
curl -X POST http://localhost:35025/api/rewards-simulator-detailed \
-H "Content-Type: application/json" \
-d '{
  "cgpLeftovers": {},
  "solarFarms": [
    {
      "farmId": "45-bb",
      "assetId": "USDG",
      "regionId": 1,
      "netWeeklyImpactAssets": "23000000000000000000",
      "protocolDepositValue": "10000000",
      "assetsRequired": "10000000",
      "firstWeek": 96,
      "weeksAlive": 100,
      "rewardSplit": [
        {
          "walletAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
          "glowSplitPercent6Decimals": "1000000",
          "depositSplitPercent6Decimals": "1000000"
        }
      ]
    }
  ]
}'
```

### Sample Response

The response is a `SimulationDiagnostics` object containing the raw simulation output, any errors, and a detailed breakdown of competitions.

```json
{
  "output": {
    "totalRegions": 1,
    "regionalStats": [
      {
        "regionId": 1,
        "assets": ["USDG"]
      }
    ],
    "weeklyRewards": [
      {
        "weekNumber": 96,
        "perFarmRewards": [
          {
            "farmId": "45-bb",
            "assetId": "USDG",
            "regionId": 1,
            "amount": "100000"
          }
        ]
      }
    ]
  },
  "errors": [],
  "competitions": [
    {
      "regionId": 1,
      "assetId": "USDG",
      "firstWeek": 96,
      "finalWeek": 195,
      "farms": [
        {
          "farmId": "45-bb",
          "protocolDepositValue": "10000000",
          "assetsRequired": "10000000",
          "firstWeek": 96,
          "finalWeek": 195,
          "assetId": "USDG",
          "regionId": 1,
          "rewardSplits": [
            {
              "walletAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
              "glowSplitPercent6Decimals": "1000000",
              "depositSplitPercent6Decimals": "1000000"
            }
          ]
        }
      ],
      "buckets": [
        {
          "weekNumber": 96,
          "totalDeposits": "100000",
          "totalImpactAssets": "23000000000000000000",
          "poolNetAssets": "0",
          "poolNetDeposits": "0",
          "glwInflation": "120641000000000000000000",
          "firstWeekFarms": ["45-bb"],
          "ongoingFarms": [],
          "lastWeekFarms": [],
          "farmStates": [
            {
              "farmId": "45-bb",
              "depositsContributed": "100000",
              "impactAssetsContributed": "23000000000000000000",
              "accumulatedDrawdown": "100000",
              "netOverperformance": "0",
              "rewardsThisWeek": "100000"
            }
          ]
        }
      ]
    }
  ]
}
```

### Query Parameters

*   `preloadGlowV1`: A string that, if set to `true`, will merge data from `v1-data.json`. This behaves identically to the same parameter on the `/api/rewards-simulator` endpoint.
*   `week`: This parameter is accepted by the server but is ignored by this specific endpoint. The response will always contain data for all weeks.

### Response Codes and Errors

*   **200 OK**: The simulation completed successfully without any internal consistency errors. The response body will be the full `SimulationDiagnostics` object.
*   **400 Bad Request**: The input JSON failed validation. The response body will be a JSON object with an `error` key containing a descriptive message.
*   **422 Unprocessable Entity**: The simulation ran to completion, but internal consistency checks failed. The response body will be the full `SimulationDiagnostics` object, with the `errors` array populated with messages describing the issues.
*   **500 Internal Server Error**: An unexpected internal error occurred. The response body will contain a JSON object with an `error` key.

---

## `POST /api/rewards-simulator-multi-asset`

### Description

This endpoint extends the rewards simulator to support farms with protocol deposits distributed across multiple assets (USDG, GLW, and sGCTL). Unlike the original endpoints where each farm uses a single asset type, this endpoint allows farms to split their protocol deposits across multiple assets, with each asset deposit competing independently in its corresponding competition (Region + Asset combination).

This endpoint is designed for Phase I V2 multi-asset delegation and maintains backward compatibility by leaving the existing `/api/rewards-simulator` and `/api/rewards-simulator-detailed` endpoints unchanged.

### Key Behavioral Differences

*   **Multi-Asset Deposits**: Each farm can specify deposits in multiple assets (USDG, GLW, and/or sGCTL)
*   **Multi-Competition Participation**: Each asset deposit competes independently in its corresponding competition
*   **Proportional Impact Distribution**: Impact assets are distributed across competitions proportionally to each asset's dollar value contribution
*   **Unified GLW Inflation**: GLW inflation rewards are calculated once per farm based on total protocol deposit value
*   **Enhanced Output Format**: Farm rewards include separate earned amounts for each asset type

### Request Body Parameters

The JSON request body has the following top-level fields:

*   `cgpLeftovers` (optional): Same as original endpoint - a map where keys are week numbers (as strings) and values are the amount of USDG to be added to that week's CGP competition.
*   `solarFarms` (required): An array of `MultiAssetSolarFarm` objects. Each object describes a farm with deposits potentially split across multiple assets.
*   `gctlDistribution` (optional): Same as original endpoint - regional GCTL staking distribution.
*   `outputFarms` (optional): Same as original endpoint - filter results to specific farm IDs.

#### MultiAssetSolarFarm Structure

Each farm in the `solarFarms` array has the following fields:

*   `farmId` (required): Unique identifier for the farm
*   `regionId` (required): The region where the farm operates (1 = CGP, 2 = Utah, etc.)
*   `netWeeklyImpactAssets` (required): Weekly carbon credit production (scaled by 1e18)
*   `firstWeek` (required): First week the farm is active
*   `weeksAlive` (required): Number of weeks the farm participates (minimum 2)
*   `totalProtocolDepositValue` (required): Total dollar value of all asset deposits combined (scaled by 1e6)
*   `assets` (required): Array of asset deposits (must contain at least one)
*   `rewardSplit` (required): Array defining how rewards are distributed to wallets (uses existing structure)

#### Asset Structure

Each element in the `assets` array specifies a deposit in one asset type:

*   `assetId` (required): The asset identifier - `"USDG"`, `"GLW"`, or `"SGCTL"`
*   `assetsRequired` (required): Actual amount of the asset deposited. Scaling depends on asset:
    *   USDG: scaled by 1e6
    *   GLW: scaled by 1e18
    *   SGCTL: scaled by 1e6
*   `assetsRequiredUSDC` (required): Dollar value of this asset deposit (scaled by 1e6)
*   `quotedByGVEPricePerAsset` (required): Asset price in USD as quoted by GVE (scaled by 1e6). Example: $1.23 becomes `"1230000"`

#### RewardSplit Structure

The `rewardSplit` array uses the existing structure from the original endpoint. Each wallet's `depositSplitPercent6Decimals` applies uniformly to rewards from all asset types.

### Sample Input

```json
{
  "cgpLeftovers": {},
  "solarFarms": [
    {
      "farmId": "farm-abc123",
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
          "quotedByGVEPricePerAsset": "400000"
        },
        {
          "assetId": "SGCTL",
          "assetsRequiredUSDC": "75000000000",
          "assetsRequired": "75000000000",
          "quotedByGVEPricePerAsset": "1000000"
        },
        {
          "assetId": "USDG",
          "assetsRequiredUSDC": "75000000000",
          "assetsRequired": "75000000000",
          "quotedByGVEPricePerAsset": "1000000"
        }
      ],
      "rewardSplit": [
        {
          "walletAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
          "glowSplitPercent6Decimals": "600000",
          "depositSplitPercent6Decimals": "600000"
        },
        {
          "walletAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
          "glowSplitPercent6Decimals": "400000",
          "depositSplitPercent6Decimals": "400000"
        }
      ]
    }
  ],
  "gctlDistribution": {
    "3": "50000000000000000000000"
  }
}
```

### Example Code for Calling API

```bash
curl -X POST http://localhost:35025/api/rewards-simulator-multi-asset \
-H "Content-Type: application/json" \
-d '{
  "cgpLeftovers": {},
  "solarFarms": [
    {
      "farmId": "farm-abc123",
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
          "quotedByGVEPricePerAsset": "400000"
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
  ]
}'
```

### Sample Response

The response structure differs from the original endpoint to accommodate multiple asset rewards per farm per week:

```json
{
  "102": {
    "walletDistributions": [
      {
        "userAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
        "assetsEarned": {
          "GLW": "738000000000000000000",
          "SGCTL": "270000000",
          "USDG": "150000000"
        },
        "glowInflationEarned": "540000000000000000000",
        "traces": [
          {
            "farmId": "farm-abc123",
            "assetId": "GLW",
            "amount": "738000000000000000000",
            "regionId": 3,
            "inflationRewardSplit6Decimals": "600000",
            "depositRewardSplit6Decimals": "600000",
            "glowInflationReward": "540000000000000000000"
          },
          {
            "farmId": "farm-abc123",
            "assetId": "SGCTL",
            "amount": "270000000",
            "regionId": 3,
            "inflationRewardSplit6Decimals": "0",
            "depositRewardSplit6Decimals": "600000",
            "glowInflationReward": "0"
          },
          {
            "farmId": "farm-abc123",
            "assetId": "USDG",
            "amount": "150000000",
            "regionId": 3,
            "inflationRewardSplit6Decimals": "0",
            "depositRewardSplit6Decimals": "600000",
            "glowInflationReward": "0"
          }
        ]
      },
      {
        "userAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
        "assetsEarned": {
          "GLW": "492000000000000000000",
          "SGCTL": "180000000",
          "USDG": "100000000"
        },
        "glowInflationEarned": "360000000000000000000",
        "traces": [
          {
            "farmId": "farm-abc123",
            "assetId": "GLW",
            "amount": "492000000000000000000",
            "regionId": 3,
            "inflationRewardSplit6Decimals": "400000",
            "depositRewardSplit6Decimals": "400000",
            "glowInflationReward": "360000000000000000000"
          },
          {
            "farmId": "farm-abc123",
            "assetId": "SGCTL",
            "amount": "180000000",
            "regionId": 3,
            "inflationRewardSplit6Decimals": "0",
            "depositRewardSplit6Decimals": "400000",
            "glowInflationReward": "0"
          },
          {
            "farmId": "farm-abc123",
            "assetId": "USDG",
            "amount": "100000000",
            "regionId": 3,
            "inflationRewardSplit6Decimals": "0",
            "depositRewardSplit6Decimals": "400000",
            "glowInflationReward": "0"
          }
        ]
      }
    ],
    "farmRewards": [
      {
        "id": "farm-abc123-week-102",
        "farmId": "farm-abc123",
        "weekIndex": 102,
        "regionId": 3,
        "assets": [
          {
            "assetId": "GLW",
            "assetEarned": "1230000000000000000000"
          },
          {
            "assetId": "SGCTL",
            "assetEarned": "450000000"
          },
          {
            "assetId": "USDG",
            "assetEarned": "250000000"
          }
        ],
        "glowInflationReward": "900000000000000000000",
        "protocolDeposit": "250000000000",
        "expectedProduction": "1570000000000000000000"
      }
    ],
    "regionData": {
      "3": {
        "GLW": {
          "protocolDepositSum": "100000000000",
          "carbonCreditProductionSum": "628000000000000000000"
        },
        "SGCTL": {
          "protocolDepositSum": "75000000000",
          "carbonCreditProductionSum": "471000000000000000000"
        },
        "USDG": {
          "protocolDepositSum": "75000000000",
          "carbonCreditProductionSum": "471000000000000000000"
        }
      }
    },
    "warnings": []
  }
}
```

### Output Format Details

#### Farm Rewards Structure

Each farm reward in the `farmRewards` array contains:

*   `id` (string): Composite identifier in format `"{farmId}-week-{weekIndex}"`
*   `farmId` (string): Original farm identifier
*   `weekIndex` (number): Week number
*   `regionId` (number): Region identifier
*   `assets` (array): Array of asset rewards, each containing:
    *   `assetId` (string): "GLW", "SGCTL", or "USDG"
    *   `assetEarned` (string): Amount earned in this asset (scaled per asset type)
*   `glowInflationReward` (string): Total GLW inflation earned by this farm (scaled by 1e18)
*   `protocolDeposit` (string): Total protocol deposit value across all assets (scaled by 1e6)
*   `expectedProduction` (string): The farm's `netWeeklyImpactAssets` value (scaled by 1e18)

#### Wallet Distributions

*   Each wallet can have multiple trace entries per farm (one per asset type deposited)
*   The `glowInflationEarned` for a wallet is calculated once based on the farm's total deposit value and the wallet's `glowSplitPercent6Decimals`
*   Asset rewards in traces are split according to each wallet's `depositSplitPercent6Decimals`, applied uniformly across all asset types

#### Region Data

The `regionData` object shows aggregated data per competition (Region + Asset combination). Impact assets are distributed proportionally:

```
impact_for_competition = netWeeklyImpactAssets × (asset_deposit_usd / total_deposit_usd)
```

For example, a farm with $100k total deposit ($40k GLW, $30k SGCTL, $30k USDG) and 1000 impact assets contributes:
*   400 impact assets to Region-GLW competition
*   300 impact assets to Region-SGCTL competition
*   300 impact assets to Region-USDG competition

### Query Parameters

Same as original endpoint:

*   `preloadGlowV1`: Set to `"true"` to merge V1 historical data
*   `week`: Specify a single week number to return data for only that week

### Response Codes and Errors

*   **200 OK**: The simulation completed successfully without any internal consistency errors.
*   **400 Bad Request**: The input JSON failed validation. Response body contains error details.
*   **404 Not Found**: Returned when the `week` query parameter is used and the requested week does not exist.
*   **422 Unprocessable Entity**: The simulation ran but internal consistency checks failed. Response includes both errors and output.
*   **500 Internal Server Error**: An unexpected server error occurred.

### Validation Rules

All original validation rules apply, plus:

*   Each farm must have at least one asset in the `assets` array
*   `assetId` must be one of: `"USDG"`, `"GLW"`, or `"SGCTL"`
*   Asset amounts must use correct scaling: USDG and SGCTL use 1e6, GLW uses 1e18
*   `totalProtocolDepositValue` is the authoritative total (not validated against sum of `assetsRequiredUSDC`)
*   `quotedByGVEPricePerAsset` must be provided for each asset
*   For each farm's `rewardSplit` array:
    *   The sum of all `glowSplitPercent6Decimals` must equal `1000000`
    *   The sum of all `depositSplitPercent6Decimals` must equal `1000000`
    *   Each wallet's split percentages apply uniformly to all asset types

---

## API Idiosyncrasies

The `rewards-simulator` API has several conventions and behaviors that are important to understand for correct integration.

*   **Numeric Values as Strings**: All large integer values (internally represented as `BigInt`) are serialized as strings in JSON requests and responses. This is to prevent precision loss that can occur with standard JSON number types, which are often parsed as floating-point numbers. Callers should be prepared to handle these numeric strings.

*   **Scaling Factors**: To perform calculations with high precision and avoid floating-point arithmetic, the API expects monetary and token values to be scaled up by a fixed factor.
    *   Dollar-denominated values (like `protocolDepositValue`) are scaled by 1e6 (e.g., $100 is sent as `"100000000"`).
    *   Most token values (like `assetsRequired` for GLW and `netWeeklyImpactAssets`) are scaled by 1e18.
    *   Notable exceptions scaled by 1e6:
        *   `assetsRequired` for the `USDG` asset
        *   `assetsRequired` for the `SGCTL` asset (staked GCTL)
    The caller is responsible for applying the correct scaling factor to all input values.

*   **Input Field Aliases**: For backward compatibility, the `netWeeklyImpactAssets` field has two aliases: `weeklyImpactAssets` and `weeklyCarbonCredits`. The API will correctly interpret any of these three names.

*   **Reward Distribution Logic**: The `rewardSplit` array is the sole mechanism for defining how a farm's rewards are distributed.
    *   The `rewardSplit` array is a **required** field for each `SolarFarm` object in the input. It must contain at least one entry.
    *   The sum of all `glowSplitPercent6Decimals` values within a farm's `rewardSplit` array must be exactly `1000000`.
    *   The sum of all `depositSplitPercent6Decimals` values within a farm's `rewardSplit` array must also be exactly `1000000`.
    *   The legacy `rewardsAddress` field is no longer supported.

*   **Error Reporting with Full Output**: When a `422 Unprocessable Entity` response is returned, it indicates that the simulation completed but failed internal consistency checks. Both API endpoints will still provide the full computed output alongside the array of error messages. This allows developers to inspect the final state of the simulation to help diagnose the cause of the consistency issue.