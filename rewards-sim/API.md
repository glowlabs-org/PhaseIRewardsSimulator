# Rewards Simulator HTTP API

This document explains all HTTP API endpoints exposed by the rewards-simulator server, their inputs, outputs, error codes, and usage examples with curl and Python.

Server defaults
- Host: 127.0.0.1
- Port: 35025
- Base URL: http://127.0.0.1:35025

Content type
- All API requests and responses use application/json.
- All big integers must be encoded as JSON strings in both inputs and outputs.



## Endpoints Overview

- POST /api/rewards-simulator
  - Returns the public rollup format used by downstream systems.
  - Supports query params:
    - preloadGlowV1=true (optional)
    - week=NUMBER (optional; when present, returns a single week object instead of the full map)
- POST /api/rewards-simulator-detailed
  - Returns full internal diagnostics and detailed competition state.
  - Supports query params:
    - preloadGlowV1=true (optional)
- POST /ui/rewards-simulator-detailed
  - Alias of /api/rewards-simulator-detailed (same behavior and schema).

Non-API static routes exist (/, /index.html, /assets/*, /js/*, etc.) to serve the visualization UI; they are not covered here.



## Input schema (request body)

Top-level object
- cgpLeftovers: object mapping weekNumber (u64) to amount (BigInt as string)
- solarFarms: array of SolarFarm objects

SolarFarm object (camelCase on the wire)
- farmId: string (non-empty)
- assetId: string (e.g., "usdg", "glw")
- regionId: string or number
  - If number: 1 maps to "cgp", 2 maps to "utah", other numbers remain numeric in outputs where strings are used upstream.
- netWeeklyImpactAssets: string BigInt (scaled; typically 1e18)
- protocolDepositValue: string BigInt (scaled dollars; 1e6)
- assetsRequired: string BigInt
  - For "usdg": typically scaled 1e6
  - Otherwise: typically scaled 1e18
- rewardsAddress: string Ethereum address (optional) 0x-prefixed (20-byte hex). Ignored if rewardSplit is present.
- rewardSplit: array (optional; when present it must satisfy invariants)
  - walletAddress: string Ethereum address
  - glowSplitPercent6Decimals: string BigInt; non-negative; sums to 1_000_000 across all splits
  - depositSplitPercent6Decimals: string BigInt; non-negative; sums to 1_000_000 across all splits
- firstWeek: integer > 0 and < 2^12
- weeksAlive: integer >= 2 and <= 2^12

BigInt JSON encoding
- All big integer fields must be encoded as strings in both request and response JSON (e.g., "1000000000000000000").



## Output schemas

### A) Public rollup format (/api/rewards-simulator)

When no week query param is provided:
- Returns an object mapping "weekNumberString" -> PublicWeekOutput.

When week=NUMBER is provided:
- Returns a single PublicWeekOutput object (not wrapped in a week map).
- If the week is missing: HTTP 404 with {"error": "..."}.

PublicWeekOutput
- walletDistributions: array of WalletDistribution
  - userAddress: string
  - assetsEarned: object mapping assetId -> string BigInt total earned this week
  - glowInflationEarned: string BigInt total GLW inflation allocated to that wallet this week
  - traces: array of WalletTrace
    - farmId: string
    - asset: string
    - inflationRewardSplit6Decimals: string BigInt
    - depositRewardSplit6Decimals: string BigInt
    - amount: string BigInt (asset amount from this farm to this wallet)
    - regionId: number or string (1 for CGP, 2 for Utah, or string for others)
    - glowInflationReward: string BigInt
- farmRewards: array of FarmRewardOut
  - id: string (farmId)
  - asset: string
  - regionId: number or string
  - assetEarned: string BigInt
  - glowInflationReward: string BigInt
  - protocolDeposit: string BigInt
  - expectedProduction: string BigInt (alias of weekly impact assets)
- regionData: object mapping regionId (number or string) -> { assetId -> RegionAssetSummary }
  - protocolDepositSum: string BigInt
  - carbonCreditProductionSum: string BigInt (alias of impact assets)
- warnings: array of strings (consistency warnings if any)

Special behavior:
- If consistency warnings occur, the endpoint returns HTTP 422 with body {"errors": [...], "output": PublicWeekOutputMapOrSingleWeek}.
- If a week query is provided and exists, the same 422 rule applies with the single-week object embedded under "output".
- If a week query is provided but not found, returns HTTP 404 with {"error": "requested week not found: NUMBER"}.

### B) Detailed format (/api/rewards-simulator-detailed and /ui/rewards-simulator-detailed)

Returns SimulationDiagnostics:
- output: OutputData (summary per week and per farm; small, public-facing summary, not the same shape as A)
  - totalRegions: number
  - regionalStats: array of { region: string, assets: string[] }
  - weeklyRewards: array of { weekNumber: u64, perFarmRewards: [{ farmId, assetId, regionId, amount(BigInt string), rewardsAddress? }] }
- errors: array of string (consistency errors/warnings)
- competitions: array of DetailedCompetition
  - regionId: string
  - assetId: string
  - firstWeek: u64
  - finalWeek: u64
  - farms: array of DetailedFarmInfo (per-farm static info and reward splits)
  - buckets: array of DetailedBucket (per-week, per-competition state)
    - weekNumber: u64
    - totalDeposits: string BigInt
    - totalImpactAssets: string BigInt
    - poolNetAssets: string BigInt
    - poolNetDeposits: string BigInt
    - glwInflation: string BigInt
    - firstWeekFarms | ongoingFarms | lastWeekFarms: string[] (farm IDs by role this week)
    - farmStates: array of DetailedFarmBucketState (per-farm dynamic state for the week)
      - depositsContributed: string BigInt
      - impactAssetsContributed: string BigInt
      - accumulatedDrawdown: string BigInt
      - netOverperformance: string BigInt
      - rewardsThisWeek: string BigInt

Special behavior:
- If consistency warnings occur, returns HTTP 422 with SimulationDiagnostics (still contains full data).
- For successful runs without consistency warnings, returns HTTP 200 with SimulationDiagnostics.



## Query parameters

- preloadGlowV1=true
  - When present, merges in farms and CGP leftovers from v1-data.json on disk. Duplicate farm IDs between payload and v1 cause a 400.
- week=NUMBER (only supported on /api/rewards-simulator)
  - Filters the response to a single week object and returns 404 if that week is missing.



## Error model

HTTP 200 OK
- Successful request with no consistency warnings.

HTTP 400 Bad Request
- Input validation errors such as:
  - No farms provided
  - Empty farmId or assetId/regionId
  - Invalid rewardsAddress or rewardSplit walletAddress (non-Ethereum address)
  - firstWeek <= 0 or >= 2^12
  - weeksAlive < 2 or > 2^12
  - Negative or zero values for protocolDepositValue or assetsRequired
  - Reward split percentages negative or exceeding 1_000_000
  - Reward split sums not equal to 1_000_000 for each split category
  - Duplicate farm IDs (including when merging v1)

HTTP 404 Not Found
- When week query is present and the requested week is not found in the public rollup output.

HTTP 422 Unprocessable Entity
- Internal consistency warnings detected during computation. The response still contains full outputs:
  - /api/rewards-simulator: {"errors": [...], "output": {...}} (map or single-week)
  - /api/rewards-simulator-detailed: SimulationDiagnostics (with errors != empty)

HTTP 500 Internal Server Error
- Unexpected internal errors.



## Usage examples (curl)

Note: Use application/json and encode BigInt fields as strings.

Minimal example payload (one farm, two weeks)

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

1) Public rollup (all weeks)
```
curl -sS -H 'content-type: application/json' \
  -X POST \
  -d @payload.json \
  http://127.0.0.1:35025/api/rewards-simulator | jq .
```

2) Public rollup for a single week (week=96)
```
curl -sS -H 'content-type: application/json' \
  -X POST \
  -d @payload.json \
  'http://127.0.0.1:35025/api/rewards-simulator?week=96' | jq .
```

3) Public rollup with v1 preload
```
curl -sS -H 'content-type: application/json' \
  -X POST \
  -d @payload.json \
  'http://127.0.0.1:35025/api/rewards-simulator?preloadGlowV1=true' | jq .
```

4) Detailed results (diagnostics + full internal state)
```
curl -sS -H 'content-type: application/json' \
  -X POST \
  -d @payload.json \
  http://127.0.0.1:35025/api/rewards-simulator-detailed | jq .
```

5) Detailed results with v1 preload (alias route)
```
curl -sS -H 'content-type: application/json' \
  -X POST \
  -d @payload.json \
  'http://127.0.0.1:35025/ui/rewards-simulator-detailed?preloadGlowV1=true' | jq .
```



## Usage examples (Python)

Assumes: pip install requests

Common helpers
```python
import requests
import json

BASE = "http://127.0.0.1:35025"

payload = {
  "cgpLeftovers": {},
  "solarFarms": [
    {
      "farmId": "A",
      "assetId": "usdg",
      "regionId": 2,
      "netWeeklyImpactAssets": "1000000000000000000",  # 1e18
      "protocolDepositValue": "10000000",              # $10 (1e6 scale)
      "assetsRequired": "10000000",                    # 10 usdg (1e6 scale)
      "rewardsAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
      "firstWeek": 96,
      "weeksAlive": 2
    }
  ]
}
headers = {"content-type": "application/json"}
```

1) Public rollup, full map
```python
r = requests.post(f"{BASE}/api/rewards-simulator", headers=headers, data=json.dumps(payload))
if r.status_code == 200:
    data = r.json()  # dict of weekStr -> PublicWeekOutput
elif r.status_code == 422:
    diag = r.json()  # {"errors": [...], "output": {...}}
    print("Consistency warnings:", diag.get("errors"))
    data = diag.get("output")
else:
    print("Error:", r.status_code, r.text)
```

2) Public rollup, single week (96)
```python
r = requests.post(f"{BASE}/api/rewards-simulator?week=96", headers=headers, data=json.dumps(payload))
if r.status_code == 200:
    week_obj = r.json()  # PublicWeekOutput (single object)
elif r.status_code == 422:
    diag = r.json()  # {"errors": [...], "output": {...}} where output is the single-week object
    print("Consistency warnings:", diag.get("errors"))
    week_obj = diag.get("output")
elif r.status_code == 404:
    print("Requested week not found")
else:
    print("Error:", r.status_code, r.text)
```

3) Public rollup with v1 preload
```python
r = requests.post(f"{BASE}/api/rewards-simulator?preloadGlowV1=true", headers=headers, data=json.dumps(payload))
print(r.status_code)
print(r.json())
```

4) Detailed results (full diagnostics)
```python
r = requests.post(f"{BASE}/api/rewards-simulator-detailed", headers=headers, data=json.dumps(payload))
if r.status_code in (200, 422):
    diag = r.json()       # SimulationDiagnostics
    errors = diag.get("errors", [])
    if errors:
        print("Consistency warnings:", errors)
    competitions = diag.get("competitions", [])
    # Example: iterate competitions and weeks
    for comp in competitions:
        region = comp["regionId"]
        asset = comp["assetId"]
        for b in comp.get("buckets", []):
            wk = b["weekNumber"]
            total_deposits = b["totalDeposits"]    # BigInt string
            total_impact = b["totalImpactAssets"]  # BigInt string
            # ...
else:
    print("Error:", r.status_code, r.text)
```

5) Detailed results with v1 preload (alias)
```python
r = requests.post(f"{BASE}/ui/rewards-simulator-detailed?preloadGlowV1=true", headers=headers, data=json.dumps(payload))
print(r.status_code)
print(r.json())
```



## Input validation and constraints (summary)

- Numerical fields must be positive and non-zero, except netWeeklyImpactAssets which may be zero at the farm level (the bucket must still have non-zero total impact assets).
- firstWeek: 1..(2^12 - 1)
- weeksAlive: 2..(2^12)
- Ethereum addresses must be valid 0x-prefixed addresses.
- rewardSplit, when present:
  - All walletAddress entries must be valid Ethereum addresses.
  - Each of glowSplitPercent6Decimals and depositSplitPercent6Decimals must be 0..1_000_000.
  - Sums of each must be exactly 1_000_000.
- Duplicate farmId is not allowed (including when merging v1).
- BigInt values must be strings in JSON.

RegionId mapping
- Accepts either a string (e.g., "cgp", "utah") or a number:
  - 1 -> "cgp"
  - 2 -> "utah"
  - Other numbers are passed through as-is where applicable.



## Special behaviors

- preloadGlowV1=true merges v1-data.json
  - Adds v1 cgpLeftovers into request cgpLeftovers (per-week sums).
  - Appends v1 farms; duplicate IDs cause a 400.
- GLW weekly inflation (from GCTL) is applied per-region and proportional to total deposits in active competitions that week.
  - cgp: 120,641 GLW/week (1e18 scaled)
  - utah: 18,119 GLW/week (1e18 scaled)
  - colorado: 18,119 GLW/week (1e18 scaled)
  - missouri: 18,119 GLW/week (1e18 scaled)
- CGP leftovers bonus (cgp/usdg only) is added proportional to deposits recovered that week.
- When consistency checks detect small dust/rounding deltas (per tolerance), outputs are still returned with HTTP 422 and errors noted.



## Practical tips

- Always send application/json and BigInts as strings.
- If you only need a public downstream format for a single week, use /api/rewards-simulator?week=NUMBER to reduce payload size.
- If you need to drive a UI or debug computations, use /api/rewards-simulator-detailed (or the /ui/ alias) to get full bucket-by-bucket and farm-by-farm state.
- Handle HTTP 422 by reading and honoring the "errors" array while still consuming the "output"/SimulationDiagnostics payload.
- For USDG assets, assetsRequired is typically 1e6 scaled; for other assets, 1e18 is typical. The simulator is scale-agnostic as long as you remain consistent.



## Example minimal end-to-end flow (Python)

```python
import requests, json

BASE = "http://127.0.0.1:35025"
headers = {"content-type": "application/json"}
payload = {
  "cgpLeftovers": {},
  "solarFarms": [{
    "farmId": "F1",
    "assetId": "glw",
    "regionId": "cgp",
    "netWeeklyImpactAssets": "1000000000000000000",
    "protocolDepositValue": "10000000",
    "assetsRequired": "20000000",
    "rewardsAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
    "firstWeek": 10,
    "weeksAlive": 2
  }]
}

# Public
r = requests.post(f"{BASE}/api/rewards-simulator", headers=headers, data=json.dumps(payload))
print(r.status_code)
print(r.json())

# Detailed
r = requests.post(f"{BASE}/api/rewards-simulator-detailed", headers=headers, data=json.dumps(payload))
print(r.status_code)
diag = r.json()
print("errors:", diag.get("errors"))
print("first competition keys:", [ (c["regionId"], c["assetId"]) for c in diag.get("competitions", []) ][:1])
```



## Troubleshooting

- 400 errors: verify rewardSplit sums, address formats, weeksAlive bounds, and that BigInts are strings.
- 404 with week=NUMBER: verify that the requested week is covered by at least one active competition in your input or preload.
- 422 errors: read the "errors" array. You still receive the full data payload for inspection.
- 500 errors: inspect server logs/stderr for details.