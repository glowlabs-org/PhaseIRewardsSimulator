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

## API Idiosyncrasies

The `rewards-simulator` API has several conventions and behaviors that are important to understand for correct integration.

*   **Numeric Values as Strings**: All large integer values (internally represented as `BigInt`) are serialized as strings in JSON requests and responses. This is to prevent precision loss that can occur with standard JSON number types, which are often parsed as floating-point numbers. Callers should be prepared to handle these numeric strings.

*   **Scaling Factors**: To perform calculations with high precision and avoid floating-point arithmetic, the API expects monetary and token values to be scaled up by a fixed factor.
    *   Dollar-denominated values (like `protocolDepositValue`) are scaled by 1e6 (e.g., $100 is sent as `"100000000"`).
    *   Most token values (like `assetsRequired` for GLW and `netWeeklyImpactAssets`) are scaled by 1e18.
    *   A notable exception is `assetsRequired` for the `USDG` asset, which is scaled by 1e6.
    The caller is responsible for applying the correct scaling factor to all input values.

*   **Input Field Aliases**: For backward compatibility, the `netWeeklyImpactAssets` field has two aliases: `weeklyImpactAssets` and `weeklyCarbonCredits`. The API will correctly interpret any of these three names.

*   **Reward Distribution Logic**: The `rewardSplit` array is the sole mechanism for defining how a farm's rewards are distributed.
    *   The `rewardSplit` array is a **required** field for each `SolarFarm` object in the input. It must contain at least one entry.
    *   The sum of all `glowSplitPercent6Decimals` values within a farm's `rewardSplit` array must be exactly `1000000`.
    *   The sum of all `depositSplitPercent6Decimals` values within a farm's `rewardSplit` array must also be exactly `1000000`.
    *   The legacy `rewardsAddress` field is no longer supported.

*   **Error Reporting with Full Output**: When a `422 Unprocessable Entity` response is returned, it indicates that the simulation completed but failed internal consistency checks. Both API endpoints will still provide the full computed output alongside the array of error messages. This allows developers to inspect the final state of the simulation to help diagnose the cause of the consistency issue.