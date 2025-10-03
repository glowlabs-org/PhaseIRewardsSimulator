# Rewards Simulator HTTP API (OpenAPI 3.1)

This document provides a complete, developer‑friendly API reference for the Rewards Simulator service, including an OpenAPI 3.1 definition and practical examples. It is designed so you can:

- Understand the data model and wire formats at a glance.
- Make requests quickly (curl, JavaScript, Rust examples).
- Correctly parse responses, including the two result shapes of the public rollup endpoint.
- Handle errors and warnings deterministically.

If you prefer a machine‑readable spec, copy the OpenAPI YAML block below into your tooling. For most SDK generators, the oneOf/anyOf types here work well; if you are using stricter generators, see the “Compatibility Notes” at the end.



## Base URL and Server Behavior

- Default host: 127.0.0.1
- Default port: 35025
- Base URL: http://127.0.0.1:35025

Binding
- The server binds to 0.0.0.0:35025 by default.
- Override with env: BIND_ADDR=127.0.0.1:8080 (host:port).

Content Type
- All requests and responses: application/json.

BigInt JSON encoding
- Any “big integer” value is transmitted as a JSON string: "12345678901234567890".
- Never send floats. All arithmetic is integer and deterministic.

JSON naming
- All JSON fields on the wire (inputs and outputs) use camelCase.



## Quick Start

1) POST a simulation request
- Endpoint: POST /api/rewards-simulator
- Purpose: Get the public rollup in a compact, downstream‑friendly format.

Minimal example (one farm, two weeks):

curl
- Request:
  curl -sS -H 'content-type: application/json' -X POST \
    http://127.0.0.1:35025/api/rewards-simulator \
    -d '{
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
    }'

- Response (object map keyed by weekNumberString -> PublicWeekOutput):
  {
    "96": { ... PublicWeekOutput ... },
    "97": { ... PublicWeekOutput ... }
  }

Optional single-week view:
- Add ?week=96 to return only the PublicWeekOutput object for week 96 (not a map):
  curl -sS -H 'content-type: application/json' -X POST \
    'http://127.0.0.1:35025/api/rewards-simulator?week=96' \
    -d '{ "cgpLeftovers": {}, "solarFarms": [ ... ] }'

2) POST for detailed diagnostics (UI/debug)
- Endpoint: POST /api/rewards-simulator-detailed
- Purpose: Get full internal state: competitions, per‑week buckets, and farm states.

curl
- Request:
  curl -sS -H 'content-type: application/json' -X POST \
    http://127.0.0.1:35025/api/rewards-simulator-detailed \
    -d '{ "cgpLeftovers": {}, "solarFarms": [ ... ] }'

- Response (SimulationDiagnostics):
  {
    "output": { "totalRegions": 1, "regionalStats": [...], "weeklyRewards": [...] },
    "errors": [],
    "competitions": [ ... DetailedCompetition ... ]
  }

3) Parsing tips
- BigInt fields are always strings. Use a bignum library or language BigInt type.
- The public endpoint may return either:
  - A map of weekNumberString -> PublicWeekOutput (default), or
  - A single PublicWeekOutput (if you pass ?week=NUMBER).
  Handle both with a oneOf strategy.
- If consistency warnings occur, HTTP 422 is returned with:
  { "errors": [...], "output": <same shape as 200> }
  You can safely consume the output field even when status is 422.



## Code Examples

JavaScript (fetch)
- Public rollup (all weeks):
  const body = {
    cgpLeftovers: {},
    solarFarms: [
      {
        farmId: "A",
        assetId: "usdg",
        regionId: 2,
        netWeeklyImpactAssets: "1000000000000000000",
        protocolDepositValue: "10000000",
        assetsRequired: "10000000",
        rewardsAddress: "0xa273164a466dbF9F0173996078fb382acC73F9E3",
        firstWeek: 96,
        weeksAlive: 2
      }
    ]
  };
  const res = await fetch("http://127.0.0.1:35025/api/rewards-simulator", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body)
  });
  const data = await res.json();
  if (res.status === 200) {
    // data is a map { "96": PublicWeekOutput, ... }
  } else if (res.status === 422) {
    // data = { errors: string[], output: { ... } }
    // Use data.output
  } else {
    // data = { error: string }
  }

- Public rollup (single week):
  const res = await fetch("http://127.0.0.1:35025/api/rewards-simulator?week=96", { ... });

- Detailed diagnostics:
  const res = await fetch("http://127.0.0.1:35025/api/rewards-simulator-detailed", { ... });
  // If 200 or 422, body is SimulationDiagnostics (with .errors possibly non-empty)

Rust (reqwest)
- Public rollup:
  use reqwest::Client;
  use serde_json::json;

  #[tokio::main]
  async fn main() -> anyhow::Result<()> {
      let client = Client::new();
      let body = json!({
        "cgpLeftovers": {},
        "solarFarms": [{
          "farmId": "A",
          "assetId": "usdg",
          "regionId": 2,
          "netWeeklyImpactAssets": "1000000000000000000",
          "protocolDepositValue": "10000000",
          "assetsRequired": "10000000",
          "rewardsAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
          "firstWeek": 96,
          "weeksAlive": 2
        }]
      });

      let url = "http://127.0.0.1:35025/api/rewards-simulator?week=96";
      let resp = client.post(url).json(&body).send().await?;
      let status = resp.status();
      let value: serde_json::Value = resp.json().await?;

      match status.as_u16() {
          200 => {
              // value is PublicWeekOutput
              println!("{}", value);
          }
          422 => {
              // value: { errors: [...], output: PublicWeekOutput or map }
              println!("warnings: {}", value["errors"]);
              println!("output: {}", value["output"]);
          }
          _ => {
              eprintln!("error: {}", value["error"]);
          }
      }
      Ok(())
  }

Node (handling BigInt strings)
- Many libraries can handle arbitrarily large integers. If not, keep strings as-is or use a big integer lib (e.g., BigInt in modern JS).


## OpenAPI 3.1 Specification

Copy/paste into your favorite OpenAPI viewer or code generator.

```yaml
openapi: 3.1.0
info:
  title: Rewards Simulator API
  version: 1.0.0
  description: |
    Simulate weekly rewards for solar farms across regions and assets. All large numeric values are JSON strings.
servers:
  - url: http://127.0.0.1:35025
paths:
  /api/rewards-simulator:
    post:
      summary: Run simulation (public rollup)
      description: |
        Returns a compact public rollup suitable for downstream systems.
        - By default returns a map of weekNumberString -> PublicWeekOutput
        - If query param `week` is supplied, returns a single PublicWeekOutput object
      parameters:
        - name: preloadGlowV1
          in: query
          description: When "true", merges v1-data.json with the request.
          required: false
          schema:
            type: boolean
        - name: week
          in: query
          description: If provided, return only that week's results (single object instead of map).
          required: false
          schema:
            type: integer
            format: int64
            minimum: 1
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/InputData'
            examples:
              minimal:
                summary: Minimal one-farm, two-week input
                value:
                  cgpLeftovers: {}
                  solarFarms:
                    - farmId: A
                      assetId: usdg
                      regionId: 2
                      netWeeklyImpactAssets: "1000000000000000000"
                      protocolDepositValue: "10000000"
                      assetsRequired: "10000000"
                      rewardsAddress: "0xa273164a466dbF9F0173996078fb382acC73F9E3"
                      firstWeek: 96
                      weeksAlive: 2
      responses:
        '200':
          description: OK
          content:
            application/json:
              schema:
                oneOf:
                  - $ref: '#/components/schemas/PublicWeekMap'
                  - $ref: '#/components/schemas/PublicWeekOutput'
              examples:
                mapOfWeeks:
                  summary: Multi-week map
                  value:
                    "96": { walletDistributions: [], farmRewards: [], regionData: {}, warnings: [] }
                    "97": { walletDistributions: [], farmRewards: [], regionData: {}, warnings: [] }
        '400':
          description: Validation error (bad input)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorObject'
        '404':
          description: Requested week not found (when week=NUMBER provided)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorObject'
        '422':
          description: Consistency warnings; output is still provided
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/WarningsWithOutput'
        '500':
          description: Internal error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorObject'
  /api/rewards-simulator-detailed:
    post:
      summary: Run simulation (detailed diagnostics + full state)
      description: |
        Returns the full internal structure for debugging and visualization.
      parameters:
        - name: preloadGlowV1
          in: query
          description: When "true", merges v1-data.json with the request.
          required: false
          schema:
            type: boolean
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/InputData'
      responses:
        '200':
          description: OK (no consistency warnings)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SimulationDiagnostics'
        '400':
          description: Validation error (bad input)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorObject'
        '422':
          description: Consistency warnings were detected (structure still returned)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SimulationDiagnostics'
        '500':
          description: Internal error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorObject'
  /ui/rewards-simulator-detailed:
    post:
      summary: Alias of /api/rewards-simulator-detailed
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/InputData'
      responses:
        '200':
          description: OK
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SimulationDiagnostics'
        '422':
          description: Consistency warnings
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SimulationDiagnostics'
        '400':
          description: Validation error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorObject'
        '500':
          description: Internal error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorObject'
components:
  schemas:
    BigIntString:
      type: string
      pattern: '^-?[0-9]+$'
      description: Arbitrary precision integer encoded as a JSON string. No decimals, no separators.

    InputData:
      type: object
      required: [solarFarms]
      properties:
        cgpLeftovers:
          type: object
          description: >
            Map of weekNumber -> bonus USDG (BigIntString) used only for regionId=cgp and assetId=usdg.
            Keys must be decimal week numbers.
          propertyNames:
            pattern: '^[0-9]+$'
          additionalProperties:
            $ref: '#/components/schemas/BigIntString'
          default: {}
        solarFarms:
          type: array
          minItems: 1
          items:
            $ref: '#/components/schemas/SolarFarm'
    SolarFarm:
      type: object
      required:
        - farmId
        - assetId
        - regionId
        - netWeeklyImpactAssets
        - protocolDepositValue
        - assetsRequired
        - firstWeek
        - weeksAlive
      properties:
        farmId:
          type: string
          minLength: 1
          description: Unique across the entire request (and v1 merge if enabled).
        assetId:
          type: string
          description: Asset ticker (e.g., "usdg", "glw").
        regionId:
          description: Region identifier; accepts "cgp", "utah", or other string; also accepts 1 for cgp and 2 for utah.
          oneOf:
            - type: string
            - type: integer
        netWeeklyImpactAssets:
          $ref: '#/components/schemas/BigIntString'
        protocolDepositValue:
          $ref: '#/components/schemas/BigIntString'
        assetsRequired:
          $ref: '#/components/schemas/BigIntString'
        rewardsAddress:
          type: string
          nullable: true
          description: 0x-prefixed Ethereum address (ignored if rewardSplit provided).
        rewardSplit:
          type: array
          description: If provided, overrides rewardsAddress. Sums must equal 1_000_000 for both percent fields.
          items:
            $ref: '#/components/schemas/RewardSplit'
          default: []
        firstWeek:
          type: integer
          format: int64
          minimum: 1
          maximum: 4095
        weeksAlive:
          type: integer
          format: int64
          minimum: 2
          maximum: 4096
      description: |
        Scaling:
        - protocolDepositValue (dollars): 1e6
        - assetsRequired: 1e6 if assetId=usdg, else 1e18
        - netWeeklyImpactAssets: 1e18
        All values must be integers after scaling. No floats.
    RewardSplit:
      type: object
      required:
        - walletAddress
        - glowSplitPercent6Decimals
        - depositSplitPercent6Decimals
      properties:
        walletAddress:
          type: string
          description: 0x-prefixed Ethereum address.
        glowSplitPercent6Decimals:
          $ref: '#/components/schemas/BigIntString'
          description: Integer in [0, 1000000].
        depositSplitPercent6Decimals:
          $ref: '#/components/schemas/BigIntString'
          description: Integer in [0, 1000000].

    ErrorObject:
      type: object
      required: [error]
      properties:
        error:
          type: string

    WarningsWithOutput:
      type: object
      required: [errors, output]
      properties:
        errors:
          type: array
          items: { type: string }
        output:
          oneOf:
            - $ref: '#/components/schemas/PublicWeekMap'
            - $ref: '#/components/schemas/PublicWeekOutput'

    PublicWeekMap:
      type: object
      description: Map of weekNumberString -> PublicWeekOutput
      additionalProperties:
        $ref: '#/components/schemas/PublicWeekOutput'

    PublicWeekOutput:
      type: object
      required: [walletDistributions, farmRewards, regionData, warnings]
      properties:
        walletDistributions:
          type: array
          items:
            $ref: '#/components/schemas/WalletDistribution'
        farmRewards:
          type: array
          items:
            $ref: '#/components/schemas/FarmRewardOut'
        regionData:
          type: object
          description: Map regionId -> (assetId -> RegionAssetSummary). Keys can be "1" or "2" (CGP/Utah) or other strings.
          additionalProperties:
            type: object
            additionalProperties:
              $ref: '#/components/schemas/RegionAssetSummary'
        warnings:
          type: array
          items: { type: string }

    WalletDistribution:
      type: object
      required: [userAddress, assetsEarned, glowInflationEarned, traces]
      properties:
        userAddress:
          type: string
        assetsEarned:
          type: object
          description: Map assetId -> BigIntString
          additionalProperties:
            type: string
            pattern: '^-?[0-9]+$'
        glowInflationEarned:
          $ref: '#/components/schemas/BigIntString'
        traces:
          type: array
          items:
            $ref: '#/components/schemas/WalletTrace'

    WalletTrace:
      type: object
      required:
        - farmId
        - asset
        - inflationRewardSplit6Decimals
        - depositRewardSplit6Decimals
        - amount
        - regionId
        - glowInflationReward
      properties:
        farmId:
          type: string
        asset:
          type: string
        inflationRewardSplit6Decimals:
          $ref: '#/components/schemas/BigIntString'
        depositRewardSplit6Decimals:
          $ref: '#/components/schemas/BigIntString'
        amount:
          $ref: '#/components/schemas/BigIntString'
        regionId:
          description: Region identifier (number strings "1", "2" for CGP/Utah or other strings).
          oneOf:
            - type: integer
            - type: string
        glowInflationReward:
          $ref: '#/components/schemas/BigIntString'

    FarmRewardOut:
      type: object
      required:
        - id
        - asset
        - regionId
        - assetEarned
        - glowInflationReward
        - protocolDeposit
        - expectedProduction
      properties:
        id:
          type: string
        asset:
          type: string
        regionId:
          oneOf:
            - type: integer
            - type: string
        assetEarned:
          $ref: '#/components/schemas/BigIntString'
        glowInflationReward:
          $ref: '#/components/schemas/BigIntString'
        protocolDeposit:
          $ref: '#/components/schemas/BigIntString'
        expectedProduction:
          $ref: '#/components/schemas/BigIntString'

    RegionAssetSummary:
      type: object
      required: [protocolDepositSum, carbonCreditProductionSum]
      properties:
        protocolDepositSum:
          $ref: '#/components/schemas/BigIntString'
        carbonCreditProductionSum:
          $ref: '#/components/schemas/BigIntString'

    SimulationDiagnostics:
      type: object
      required: [output, errors, competitions]
      properties:
        output:
          $ref: '#/components/schemas/OutputData'
        errors:
          type: array
          items: { type: string }
        competitions:
          type: array
          items:
            $ref: '#/components/schemas/DetailedCompetition'

    OutputData:
      type: object
      required: [totalRegions, regionalStats, weeklyRewards]
      properties:
        totalRegions:
          type: integer
        regionalStats:
          type: array
          items:
            $ref: '#/components/schemas/RegionStats'
        weeklyRewards:
          type: array
          items:
            $ref: '#/components/schemas/WeekRewards'

    RegionStats:
      type: object
      required: [region, assets]
      properties:
        region:
          type: string
        assets:
          type: array
          items: { type: string }

    WeekRewards:
      type: object
      required: [weekNumber, perFarmRewards]
      properties:
        weekNumber:
          type: integer
        perFarmRewards:
          type: array
          items:
            $ref: '#/components/schemas/FarmReward'

    FarmReward:
      type: object
      required: [farmId, assetId, regionId, amount]
      properties:
        farmId:
          type: string
        assetId:
          type: string
        regionId:
          type: string
        amount:
          $ref: '#/components/schemas/BigIntString'
        rewardsAddress:
          type: string
          nullable: true

    DetailedCompetition:
      type: object
      required: [regionId, assetId, firstWeek, finalWeek, farms, buckets]
      properties:
        regionId:
          type: string
        assetId:
          type: string
        firstWeek:
          type: integer
        finalWeek:
          type: integer
        farms:
          type: array
          items:
            $ref: '#/components/schemas/DetailedFarmInfo'
        buckets:
          type: array
          items:
            $ref: '#/components/schemas/DetailedBucket'

    DetailedFarmInfo:
      type: object
      required:
        - farmId
        - protocolDepositValue
        - assetsRequired
        - firstWeek
        - finalWeek
        - assetId
        - regionId
        - rewardSplit
      properties:
        farmId:
          type: string
        protocolDepositValue:
          $ref: '#/components/schemas/BigIntString'
        assetsRequired:
          $ref: '#/components/schemas/BigIntString'
        firstWeek:
          type: integer
        finalWeek:
          type: integer
        rewardsAddress:
          type: string
          nullable: true
        assetId:
          type: string
        regionId:
          type: string
        rewardSplit:
          type: array
          items:
            $ref: '#/components/schemas/RewardSplit'

    DetailedBucket:
      type: object
      required:
        - weekNumber
        - totalDeposits
        - totalImpactAssets
        - poolNetAssets
        - poolNetDeposits
        - glwInflation
        - firstWeekFarms
        - ongoingFarms
        - lastWeekFarms
        - farmStates
      properties:
        weekNumber:
          type: integer
        totalDeposits:
          $ref: '#/components/schemas/BigIntString'
        totalImpactAssets:
          $ref: '#/components/schemas/BigIntString'
        poolNetAssets:
          $ref: '#/components/schemas/BigIntString'
        poolNetDeposits:
          $ref: '#/components/schemas/BigIntString'
        glwInflation:
          $ref: '#/components/schemas/BigIntString'
        firstWeekFarms:
          type: array
          items: { type: string }
        ongoingFarms:
          type: array
          items: { type: string }
        lastWeekFarms:
          type: array
          items: { type: string }
        farmStates:
          type: array
          items:
            $ref: '#/components/schemas/DetailedFarmBucketState'

    DetailedFarmBucketState:
      type: object
      required:
        - farmId
        - depositsContributed
        - impactAssetsContributed
        - accumulatedDrawdown
        - netOverperformance
        - rewardsThisWeek
      properties:
        farmId:
          type: string
        depositsContributed:
          $ref: '#/components/schemas/BigIntString'
        impactAssetsContributed:
          $ref: '#/components/schemas/BigIntString'
        accumulatedDrawdown:
          $ref: '#/components/schemas/BigIntString'
        netOverperformance:
          $ref: '#/components/schemas/BigIntString'
        rewardsThisWeek:
          $ref: '#/components/schemas/BigIntString'
```



## Endpoint Reference (Human Summary)

POST /api/rewards-simulator
- Returns:
  - 200: Public rollup as either:
    - Map: "weekNumberString" -> PublicWeekOutput (default), or
    - Single PublicWeekOutput (if ?week=N).
  - 422: { errors: [...], output: (same shape as 200) } when consistency warnings occur.
  - 400: { error: string } for validation issues (e.g., empty farms, invalid addresses, rewardSplit sums).
  - 404: { error: string } if ?week=N not present in any active simulation.
  - 500: { error: string } for unexpected errors.

Query params:
- preloadGlowV1=true (optional): Merge v1-data.json with request. Duplicate farm IDs cause 400. cgpLeftovers are summed.
- week=NUMBER (optional): Narrow result to a single week’s object.

POST /api/rewards-simulator-detailed
- Returns:
  - 200: SimulationDiagnostics with .errors empty.
  - 422: SimulationDiagnostics with .errors non‑empty (full data included).
  - 400/500: error object.

Query params:
- preloadGlowV1=true (optional). No ?week= supported.

Alias:
- /ui/rewards-simulator-detailed behaves identically to /api/rewards-simulator-detailed.



## Request and Response Field Semantics

- regionId inputs accept string or number:
  - 1 or "cgp" map to the same region.
  - 2 or "utah" map to the same region.
  - Other regions should use strings; they are preserved.

- Scaling (all integers):
  - Dollars: 1e6
  - USDG tokens: 1e6
  - Other tokens (e.g., GLW) and impact assets: 1e18

- Reward splits:
  - If rewardSplit is present, it is used; rewardsAddress is ignored.
  - If rewardSplit is omitted and rewardsAddress is present, 100% goes to rewardsAddress.
  - rewardSplit sums must be exactly 1_000_000 for both glowSplitPercent6Decimals and depositSplitPercent6Decimals.

- Consistency warnings (422):
  - The simulator detects small dust/rounding mismatches; output remains usable and complete.

- GLW inflation (per region, per week; 1e18 scaling):
  - cgp: 120,641 GLW
  - utah: 18,119 GLW
  - colorado: 18,119 GLW
  - missouri: 18,119 GLW
  Split between active competitions in that region proportional to totalDeposits for the week.

- Special case: CGP leftovers (cgp/usdg only)
  - For week w with leftovers[w] > 0, each farm’s rewardsThisWeek is increased by:
    depositsRecovered(farm, w) * leftovers[w] / totalDeposits(w).



## Error Guide

- 400 Bad Request: Validation issues (e.g., empty farm list; invalid Ethereum address; invalid weeks; negative/zero where not allowed; rewardSplit sums don’t match; duplicate farm IDs, including v1 merge).
- 404 Not Found: When week=NUMBER is provided but does not exist in any active competition.
- 422 Unprocessable Entity: Non-fatal consistency warnings. Output is returned under “output” for the public endpoint, or as SimulationDiagnostics (with non-empty errors) for the detailed endpoint.
- 500 Internal Server Error: Unexpected errors.



## Compatibility Notes (OpenAPI vs. Actual Service)

These are edge cases where server behavior is flexible and the OpenAPI model uses best-effort typing:

1) Query booleans as strings
- The server currently parses preloadGlowV1 as a string (expects "true" to enable).
- In OpenAPI we model it as boolean. Most clients send ?preloadGlowV1=true which works fine.

2) Variant response shape for /api/rewards-simulator
- The endpoint returns either a map of weeks or a single object depending on the presence of ?week. The OpenAPI schema models this with oneOf. Some generators may require manual handling.

3) regionId accepts string or number
- Both request and response may use numeric or string region identifiers. The OpenAPI schema uses oneOf for regionId. If your generator struggles, coerce regionId to a string in your model.

4) BigInt as string
- All large numeric values are strings. Generators that strictly model numbers will need manual adjustments to use strings (recommended) or custom BigInt types.

5) Map keys that are numeric strings
- PublicWeekMap uses week numbers as object keys ("96", "97", ...). Some tooling prefers arrays; keep in mind these are JSON object properties, not an array.

If you need stricter schemas for a code generator, the quickest approach is:
- Treat all BigInt fields as strings.
- Treat regionId as a string everywhere client‑side.
- Always request a single week (?week=N) if you want a single deterministic shape.
- Consider wrapping results in your own DTO for your application’s domain.


## Appendix: Minimal Working Input

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