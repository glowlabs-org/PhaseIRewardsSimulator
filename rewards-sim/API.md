# Rewards Simulator HTTP API

This document explains how to call the Rewards Simulator HTTP API with a developer-friendly, task‑oriented layout. It includes the base URL, important wire-format conventions, endpoint overviews, and detailed specs for both primary endpoints (inputs, outputs, query params, and errors).



## 1) Server URL and Port

- Default host: 127.0.0.1
- Default port: 35025
- Base URL: http://127.0.0.1:35025

Note:
- The server binds to 0.0.0.0:35025 by default. You can override the binding via the BIND_ADDR environment variable (e.g., BIND_ADDR=127.0.0.1:8080).



## 2) API idiosyncrasies and wire conventions

Content type
- All requests and responses use application/json.

BigInt JSON encoding
- Every big integer on both input and output is encoded as a JSON string (e.g., "1000000000000000000"). Do not send floats.

Naming convention
- All JSON fields (inputs and outputs) use camelCase.

Region id mapping (input convenience)
- regionId accepts either a string or a number:
  - 1 -> "cgp"
  - 2 -> "utah"
  - Any other number remains numeric in places where strings are expected upstream; the server preserves it where applicable.

Typical scaling
- Values are provided as integers with scaling handled by the caller. Common scales:
  - Dollars: 1e6
  - Tokens/impact assets: usually 1e18 (USDG as dollars uses 1e6)
- Always round down when distributing values internally; dust may be discarded.
- Determinism requires integer math and no floats.

Consistency warnings
- The simulator may detect small “dust”/rounding mismatches. When that happens, endpoints still return full outputs but with HTTP 422 and an errors list.



## 3) Endpoints overview

- POST /api/rewards-simulator
  - Returns the “public rollup” format consumed by downstream systems.
  - Query params:
    - preloadGlowV1=true (optional)
    - week=NUMBER (optional; when present, returns a single week object instead of a map)

- POST /api/rewards-simulator-detailed
  - Returns full internal diagnostics and detailed competition state.
  - Query params:
    - preloadGlowV1=true (optional)

Note: An alias exists at /ui/rewards-simulator-detailed with the same behavior as /api/rewards-simulator-detailed; no separate section is provided for it.



## 4) /api/rewards-simulator (public rollup)

This endpoint returns a compact, public-facing rollup suitable for downstream usage. It can return either:
- a map of all simulated weeks to their outputs (default), or
- a single week’s object if you pass ?week=NUMBER.

### 4.1 Example input (request body) and field reference

Example input (minimal one-farm, two-week payload):
```json
{
  "cgpLeftovers": {},
  "solarFarms": [
    {
      "farmId": "A",
      "assetId": "usdg",
      "regionId": 2,
      "netWeeklyImpactAssets": "1000000000000000000",
      "protocolDepositValue": "10000000",
      "assetsRequired": "10000000",
      "rewardsAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
      "firstWeek": 96,
      "weeksAlive": 2
    }
  ]
}
```

Field reference
- Top-level
  - cgpLeftovers: object mapping weekNumber (u64) -> BigInt string. Only used for cgp/usdg competition as a weekly bonus source.
  - solarFarms: array of SolarFarm objects.

- SolarFarm (camelCase)
  - farmId: string (non-empty, unique across all input and merged data).
  - assetId: string (e.g., "usdg", "glw").
  - regionId: string or number (1 -> "cgp", 2 -> "utah"; other numbers are preserved as-is).
  - netWeeklyImpactAssets: BigInt string (typically 1e18 scaled).
  - protocolDepositValue: BigInt string (dollars, typically 1e6 scaled).
  - assetsRequired: BigInt string
    - If assetId is "usdg": typically 1e6 scaled.
    - Otherwise: typically 1e18 scaled.
  - rewardsAddress: 0x-prefixed Ethereum address (optional). Ignored if rewardSplit is provided.
  - rewardSplit (optional): array of splits, each entry:
    - walletAddress: Ethereum address
    - glowSplitPercent6Decimals: BigInt string in [0, 1_000_000]
    - depositSplitPercent6Decimals: BigInt string in [0, 1_000_000]
    - Sums across the array must equal 1_000_000 for both glowSplitPercent6Decimals and depositSplitPercent6Decimals.
  - firstWeek: integer > 0 and < 2^12
  - weeksAlive: integer >= 2 and <= 2^12

Validation highlights
- At least one farm required.
- farmId, assetId, regionId must be non-empty.
- Ethereum addresses must be valid (checksummed not required).
- protocolDepositValue and assetsRequired must be positive.
- netWeeklyImpactAssets may be zero for a farm, but total weekly impact must be non-zero.
- Duplicate farm IDs are rejected (including when merging v1 data).

BigInt rule reminder
- All BigInt fields are JSON strings in both requests and responses.

### 4.2 Example output (response body) and field reference

When no week= is provided, response is a map: "weekNumberString" -> PublicWeekOutput.
When week=NUMBER is provided and found, response is a single PublicWeekOutput object.

Example “single-week” output (valid JSON):
```json
{
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
      "expectedProduction": "230000000000000000000"
    }
  ],
  "regionData": {
    "1": {
      "USDG": {
        "protocolDepositSum": "10000000",
        "carbonCreditProductionSum": "23000000000000000000"
      }
    },
    "2": {
      "GLW": {
        "protocolDepositSum": "6000000",
        "carbonCreditProductionSum": "45000000000000000000"
      }
    }
  },
  "warnings": []
}
```

PublicWeekOutput fields
- walletDistributions: array of per-wallet results
  - userAddress: wallet address.
  - assetsEarned: object mapping assetId -> BigInt string (tokens earned this week).
  - glowInflationEarned: BigInt string, GLW inflation earned by this wallet this week.
  - traces: array of contributions per farm
    - farmId: string
    - asset: string
    - inflationRewardSplit6Decimals: BigInt string (0..1_000_000)
    - depositRewardSplit6Decimals: BigInt string (0..1_000_000)
    - amount: BigInt string (asset amount from this farm to this wallet)
    - regionId: number (1 for CGP, 2 for Utah) or string for others
    - glowInflationReward: BigInt string (wallet’s GLW from that farm this week)
- farmRewards: array of per-farm rollups
  - id: farmId
  - asset: string
  - regionId: number or string (see above)
  - assetEarned: BigInt string (asset rewards for the farm this week)
  - glowInflationReward: BigInt string (farm’s GLW inflation this week)
  - protocolDeposit: BigInt string (original farm deposit)
  - expectedProduction: BigInt string (alias of weekly impact assets)
- regionData: object mapping regionId -> { assetId -> RegionAssetSummary }
  - RegionAssetSummary:
    - protocolDepositSum: BigInt string (sum across active farms in that week/competition)
    - carbonCreditProductionSum: BigInt string (alias of impact assets)
- warnings: array of strings (non-fatal consistency notes)

Special behavior
- If consistency warnings occur, the endpoint returns HTTP 422 with:
  {"errors": [...], "output": PublicWeekMapOrSingleWeek}
- If a week query is provided but not found, returns HTTP 404 with:
  {"error": "requested week not found: NUMBER"}

### 4.3 Query parameters

- preloadGlowV1=true (optional)
  - Merges in farms and CGP leftovers from v1-data.json on disk.
  - Duplicate farm IDs between payload and v1 cause 400.
  - cgpLeftovers per-week amounts are summed.
- week=NUMBER (optional)
  - Returns only that week’s PublicWeekOutput object.
  - 404 if the week exists in no active competition.

### 4.4 Error codes

- 200 OK
  - Successful request with no consistency warnings.
- 400 Bad Request
  - Input validation errors (empty farms, invalid addresses, invalid weeks, negative or zero values where not allowed, invalid rewardSplit sums, duplicate farm IDs, etc.).
- 404 Not Found
  - Only when week=NUMBER is provided and the requested week is absent.
- 422 Unprocessable Entity
  - Consistency warnings detected; full output is still included under "output".
- 500 Internal Server Error
  - Unexpected internal errors.



## 5) /api/rewards-simulator-detailed (diagnostics + full state)

This endpoint returns a comprehensive structure that includes compact summaries and the full internal per-week, per-farm, per-competition state, suitable for debugging, UIs, and deep inspection.

### 5.1 Example input (request body) and field reference

The input schema is identical to /api/rewards-simulator (see section 4.1).

Example:
```json
{
  "cgpLeftovers": {},
  "solarFarms": [
    {
      "farmId": "A",
      "assetId": "usdg",
      "regionId": 2,
      "netWeeklyImpactAssets": "1000000000000000000",
      "protocolDepositValue": "10000000",
      "assetsRequired": "10000000",
      "rewardsAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
      "firstWeek": 96,
      "weeksAlive": 2
    }
  ]
}
```

### 5.2 Example output (response body) and field reference

The response is a SimulationDiagnostics object.

Abbreviated example:
```json
{
  "output": {
    "totalRegions": 1,
    "regionalStats": [
      { "region": "utah", "assets": ["usdg"] }
    ],
    "weeklyRewards": [
      {
        "weekNumber": 96,
        "perFarmRewards": [
          {
            "farmId": "A",
            "assetId": "usdg",
            "regionId": "utah",
            "amount": "5000000",
            "rewardsAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3"
          }
        ]
      }
    ]
  },
  "errors": [],
  "competitions": [
    {
      "regionId": "utah",
      "assetId": "usdg",
      "firstWeek": 96,
      "finalWeek": 97,
      "farms": [
        {
          "farmId": "A",
          "protocolDepositValue": "10000000",
          "assetsRequired": "10000000",
          "firstWeek": 96,
          "finalWeek": 97,
          "rewardsAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
          "assetId": "usdg",
          "regionId": "utah",
          "rewardSplit": []
        }
      ],
      "buckets": [
        {
          "weekNumber": 96,
          "totalDeposits": "5000000",
          "totalImpactAssets": "1000000000000000000",
          "poolNetAssets": "0",
          "poolNetDeposits": "0",
          "glwInflation": "18119000000000000000000",
          "firstWeekFarms": ["A"],
          "ongoingFarms": [],
          "lastWeekFarms": [],
          "farmStates": [
            {
              "farmId": "A",
              "depositsContributed": "5000000",
              "impactAssetsContributed": "1000000000000000000",
              "accumulatedDrawdown": "5000000",
              "netOverperformance": "0",
              "rewardsThisWeek": "5000000"
            }
          ]
        }
      ]
    }
  ]
}
```

SimulationDiagnostics fields
- output: OutputData
  - totalRegions: number
  - regionalStats: array of { region: string, assets: string[] }
  - weeklyRewards: array of:
    - weekNumber: u64
    - perFarmRewards: [{ farmId, assetId, regionId, amount(BigInt string), rewardsAddress? }]
- errors: array of string (consistency notes/warnings; empty for perfect runs)
- competitions: array of DetailedCompetition
  - regionId: string
  - assetId: string
  - firstWeek: u64
  - finalWeek: u64
  - farms: array of DetailedFarmInfo (static farm metadata and reward splits; BigInts are strings)
  - buckets: array of DetailedBucket (per-week state)
    - weekNumber: u64
    - totalDeposits: BigInt string (sum of deposits in that bucket)
    - totalImpactAssets: BigInt string (sum of impact assets in that bucket)
    - poolNetAssets: BigInt string (aggregate pool state)
    - poolNetDeposits: BigInt string (aggregate pool state)
    - glwInflation: BigInt string (per-region inflation split across competitions by deposits)
    - firstWeekFarms | ongoingFarms | lastWeekFarms: string[] (role lists for the week)
    - farmStates: array of DetailedFarmBucketState
      - depositsContributed: BigInt string
      - impactAssetsContributed: BigInt string
      - accumulatedDrawdown: BigInt string
      - netOverperformance: BigInt string
      - rewardsThisWeek: BigInt string

Special note about GLW allocations
- Current weekly GLW (1e18 scaled) per region:
  - cgp: 120,641 GLW/week
  - utah: 18,119 GLW/week
  - colorado: 18,119 GLW/week
  - missouri: 18,119 GLW/week
- Each week, a region’s allocation is split across that region’s active competitions proportionally to their totalDeposits for the week.

### 5.3 Query parameters

- preloadGlowV1=true (optional)
  - Load and merge v1-data.json with the provided payload (sums cgpLeftovers; appends farms).
  - Duplicate IDs between input and v1 cause 400.

Note: week=NUMBER is not supported on this endpoint.

### 5.4 Error codes

- 200 OK
  - Successful request with no consistency warnings.
- 400 Bad Request
  - Input validation errors (see section 4.4 for the common cases).
- 422 Unprocessable Entity
  - Consistency warnings were detected. The SimulationDiagnostics is still returned with full data.
- 500 Internal Server Error
  - Unexpected internal errors.