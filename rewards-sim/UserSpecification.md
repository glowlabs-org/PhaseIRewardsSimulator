# User Specification

rewards-simulator is a rust program that takes as input a JSON object
containing a list of solar farms and other data, and produces as output a list
of rewards that each solar farm would receive on Glow V2 Phase I for each week.

## Input and Output Structure

The input to this program is a JSON object containing a list of farms that
defines all of the farms that are enrolled in the rewards program. The input
defines what week the farm joins the rewards program, and how many weeks the
farm is participating in the rewards program. The farm needs to participate in
the rewards program for an integer number of weeks between 2 and 2^12
(inclusive), which allows for farms that are being ported from V1 to V2 to
define a shorter tenure.

The asset id defines which asset is being used for the 'assetsRequired' and
rewards, and the `assetsRequired` field defines how many assets are
participating in the weeks that remain. Solar farms that are being ported from
V1 to V2 therefore should not state their whole protocol deposit, but instead
should state the amount of protocol deposit that was remaining in the V1
buckets before the solar farm was ported over.

A special field called `cgpLeftovers` is provided which defines the total
amount of residual early liquidity rewards that were remaining in the V1
buckets.

An optional field called gctlDistribution lists the specific distribution of
GCTL tokens staked to each region. If provided, this field will determine how
many GLW inflation rewards will be distributed to each region. This field will
eventually be deprecated and replaced with a field that takes all of the gctl
staking events.

Note: `gctlDistribution` is a JSON object mapping region IDs to GCTL amounts. In JSON, object keys are always strings (e.g., `"1"`, `"2"`), even though region IDs are treated as numeric values (`u64`) internally. The amounts are BigInt values encoded as strings per the system-wide BigInt encoding rules.

```json
{
  "cgpLeftovers": {
    "97": "235000000",
    "98": "367000000"
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
    "2": "35000000000000000000000",
    "3": "21000000000000000000000",
    "4": "21000000000000000000000"
  },
  "outputFarms": [
    "45-bb"
  ]
}
```

Note: the `cgpLeftovers` is a map from week number to the amount of usdg that
was put into the corresponding bucket by the early liquidity contract. This map
is used to distribute bonus rewards to solar farms participating in the cgp
region with the usdg asset.

Note: The reward split that gets provided isn't used during computation, it's
used after the rewards for each farm are computed. The reward split array
establishes a list of wallets that will be receiving rewards, and it shows what
percentage of the glow inflation and competition rewards each address will
receive. The sum of all "glowSplitPercent6Decimals" values within a rewardSplit
array must be 1000000. The sum of all "depositSplitPercent6Decimals" values
within a rewardSplit array must also be 1000000. Every solar farm must have
reward splits, as the final output cannot be constructed without them.

Note: for the regionId, a '1' means the region is the cgp, and a '2' means the
region is utah.

The output will be a JSON object that contains all of the rewards that will be
distributed to each solar farm in each week. The final output map is
integrating with a different system, so a bunch of the variable names are
adjusted from the internal names of the rewards script.

```json
{
  "97": {
    "walletDistributions": [
      {
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
        ],
        "userAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D"
      },
      {
        "assetsEarned": {
           "GLW": "11000000000000000000"
        },
        "glowInflationEarned": "315000000000000000000",
        "traces": [
          {
             "farmId": "90-fa",
             "asset": "GLW",
             "inflationRewardSplit6Decimals": "1000000",
             "depositRewardSplit6Decimals": "1000000",
             "amount": "11000000000000000000",
             "regionId": 2,
             "glowInflationReward": "315000000000000000000"
          }
        ],
        "userAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3"
      }
    ],
    "farmRewards": [
      {
        "assetEarned": "95000",
        "glowInflationReward": "498000000000000000000",
        "id": "45-bb",
        "asset": "USDG",
        "regionId": 1,
        "protocolDeposit": "10000000",
        "expectedProduction": "230000000000000000000"
      },
      {
        "assetEarned": "11000000000000000000",
        "glowInflationReward": "315000000000000000000",
        "id": "90-fa",
        "asset": "GLW",
        "regionId": 2,
        "protocolDeposit": "6000000",
        "expectedProduction": "45000000000000000000"
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
    "warnings": [
      "this is an example warning"
    ]
  }
}
```

The output object is a map from week number to the reward distribution for that
week. The reward distribution is broken into two categories, each with
redundant data. The walletDistributions explain how all of the rewards were
distributed on a per-wallet level. And the "farmRewards" explain how all of the
rewards were distributed on a per-farm level.

Note: 'carbonCreditProductionSum' is the sum of all 'netWeeklyImpactAssets'
values for the competition. "impact assets" is more correct, but "carbon
credits" is a leftover from a legacy system and so it is used here.
"expectedProduction" is also an alias of 'netWeeklyImpactAssets'.

Note: If the asset is itself "GLW", the wallet will be recording two different
types of GLW rewards. They should be kept separate.

Note: Warnings are only used when the algorithm experiences unexpected errors
or fails consistency checks. Input validation errors result in an immediate
error.

Note: "outputFarms" is an optional input field. If provided, the API will only
produce the "farmRewards" output array and the "warnings" array, and the
"farmRewards" array will only include the farms that were mentioned by ID in
the field. The example above ignores that field for the purposes of
illustrating the full output, however had that field been honored the output
would have looked like this:

Note: "gctlDistribution" is an optional input field. The example above ignores
that field for the purposes of illustration.

```json
{
  "97": {
    "farmRewards": [
      {
        "assetEarned": "95000",
        "glowInflationReward": "498000000000000000000",
        "id": "45-bb",
        "asset": "USDG",
        "regionId": 1,
        "protocolDeposit": "10000000",
        "expectedProduction": "230000000000000000000"
      }
    ],
    "warnings": [
      "this is an example warning"
    ]
  }
}
```

## Multi-Asset Protocol Deposits (Phase I V2)

### Overview

The multi-asset endpoint `/api/rewards-simulator-multi-asset` extends the simulator to support farms where protocol deposits are distributed across multiple asset types: USDG, GLW, and sGCTL (staked GCTL). This allows a single farm to participate in multiple asset competitions simultaneously, with each asset deposit competing independently while rewards are aggregated back to the original farm.

### Multi-Competition Participation

A farm with multiple asset deposits participates in multiple competitions:

+ Each asset deposit competes in its corresponding competition identified by (Region, Asset)
+ For example, a farm in Region 3 with deposits in GLW, USDG, and sGCTL participates in three competitions: Region3-GLW, Region3-USDG, and Region3-sGCTL
+ Each competition participation maintains independent progressive vault state
+ Results from all competition participations are aggregated back to the original farm for output

This architecture preserves the existing single-asset competition logic while enabling farms to diversify their protocol deposits across multiple assets.

### Impact Asset Distribution

When a farm participates in multiple competitions, its total `netWeeklyImpactAssets` must be distributed proportionally across those competitions based on the dollar value of each asset deposit:

```
impact_for_competition = netWeeklyImpactAssets × (asset_deposit_usd / total_deposit_usd)
```

**Example:**

A farm in Region 3 produces 24 impact assets per week and has a total protocol deposit of $12:
+ $6 in USDG (50% of total)
+ $3 in GLW (25% of total)
+ $3 in sGCTL (25% of total)

The impact assets are distributed as:
+ Region3-USDG competition receives: 24 × (6/12) = 12 impact assets
+ Region3-GLW competition receives: 24 × (3/12) = 6 impact assets
+ Region3-sGCTL competition receives: 24 × (3/12) = 6 impact assets

This proportional distribution ensures that each competition receives impact assets commensurate with the dollar value of deposits competing in that competition.

#### Dust Handling in Impact Asset Distribution

When distributing `netWeeklyImpactAssets` across multiple asset deposits, the system uses integer division which rounds down (floor):

```
virtual_farm_impact = (netWeeklyImpactAssets × assetsRequiredUSDC) / totalProtocolDepositValue
```

Following the general dust handling policy described in the "Precision and Rounding" section, any remainder from this division is **discarded as dust**. This means:

dust loss is negligible at the scale of actual operations and maintains the invariant that total impact assets distributed never exceeds actual production.

### GLW Inflation Distribution and Reward Splits

GLW inflation rewards are distributed at the competition level, with each virtual sub-farm receiving inflation proportional to its participation in that specific competition. Each virtual sub-farm then applies its own asset-specific reward splits to distribute those rewards to wallet addresses.

**Distribution Process:**

1. **Competition-Level Distribution**: Each competition receives a portion of the region's GLW inflation based on total deposits
   + A region's GLW inflation is divided among its competitions proportional to each competition's `total_deposits` (denominated in USDC)
   + Example: If Region 3 has 10,000 GLW and two competitions with deposits of $100 USDC and $900 USDC respectively, they receive 1,000 and 9,000 GLW respectively

2. **Virtual Sub-Farm Distribution**: Within each competition, GLW inflation is distributed to virtual sub-farms proportionally
   + Each virtual sub-farm receives: `competition_glw_inflation × sub_farm_deposits_usdc / competition_total_deposits_usdc`
   + This is standard competition behavior - farms earn inflation based on their deposit contribution (measured in USDC value)

3. **Asset-Specific Reward Splits**: Each virtual sub-farm has its own reward split configuration
   + When a multi-asset farm is broken into virtual sub-farms, each virtual sub-farm inherits the reward split from its corresponding asset deposit
   + Example: A farm with sGCTL, USDG, and GLW deposits creates three virtual sub-farms with potentially different reward splits
   + The sGCTL virtual sub-farm might split rewards 80/20 to wallets A/B
   + The USDG virtual sub-farm might split rewards 70/30 to wallets C/D
   + The GLW virtual sub-farm might split rewards 65/35 to wallets E/F

4. **Wallet Distribution**: Each virtual sub-farm applies its reward splits to both asset rewards and GLW inflation
   + Asset rewards: `rewards_this_week × deposit_split_percent_6_decimals / 1,000,000`
   + GLW inflation: `sub_farm_glw_inflation × glow_split_percent_6_decimals / 1,000,000`
   + Note: Split percentages are stored as integers out of 1,000,000 (6 decimals of precision)
     - 100% = 1,000,000
     - 60% = 600,000
     - 5% = 50,000
   + Example: If `rewards_this_week = 1000` and `deposit_split_percent_6_decimals = 600000` (60%), the wallet receives `1000 × 600000 / 1000000 = 600` tokens
   + Each wallet address accumulates rewards from all farms (and virtual sub-farms) that reference it

**Key Insight**: Different protocol deposit assets have independent reward splits because each virtual sub-farm competes in a different competition with different competitive dynamics, leading to different performance and reward potential.

Each asset deposit creates a virtual sub-farm that competes independently:
+ **sGCTL virtual sub-farm** competes in Region3-sGCTL competition
+ **USDG virtual sub-farm** competes in Region3-USDG competition  
+ **GLW virtual sub-farm** competes in Region3-GLW competition

These competitions have different levels of competition (different numbers of farms, different performance levels). The sGCTL competition might be less crowded and more lucrative, while the GLW competition might be more saturated. This means:

+ Each virtual sub-farm recovers deposits at different rates based on its competitiveness in its specific competition
+ Each virtual sub-farm earns different amounts of asset rewards based on how it performs relative to competitors in that competition pot
+ The reward splits can reflect these different competitive dynamics - investors/operators might want different distribution terms based on which competition pot they're entering

For example, if the sGCTL competition is highly lucrative (80% deposit recovery expected), an investor might demand an 80/20 split favoring them. But if the GLW competition is more competitive (only 60% deposit recovery expected), the operator might negotiate a 50/50 split to compensate for the higher risk.

### Multi-Asset Data Structures

The multi-asset endpoint introduces new data structures:

```rs
pub struct MultiAssetSolarFarm {
    pub farm_id: String,
    pub region_id: u64,
    pub net_weekly_impact_assets: BigInt,
    pub total_protocol_deposit_value: BigInt,
    pub assets: Vec<AssetRequirement>,
    pub first_week: u64,
    pub weeks_alive: u64,
}

pub struct AssetRequirement {
    pub asset_id: String,
    pub assets_required: BigInt,
    pub assets_required_usdc: BigInt,
    pub quoted_by_gve_price_per_asset: BigInt,
    pub reward_split: Vec<RewardSplit>,
}
```

The `RewardSplit` structure remains unchanged from the single-asset model. However, unlike single-asset farms, multi-asset farms specify reward splits **per asset** rather than per farm. Each asset's reward split defines how that specific asset deposit's rewards are distributed to wallet addresses.

#### Input Example

Here's an example of a multi-asset farm with different reward splits per asset:

```json
{
  "solarFarms": [
    {
      "farmId": "farm-multi-123",
      "regionId": 3,
      "netWeeklyImpactAssets": "24000000000000000000",
      "totalProtocolDepositValue": "12000000",
      "firstWeek": 102,
      "weeksAlive": 52,
      "assets": [
        {
          "assetId": "SGCTL",
          "assetsRequired": "3000000",
          "assetsRequiredUSDC": "6000000",
          "quotedByGVEPricePerAsset": "2000000"
        },
        {
          "assetId": "USDG",
          "assetsRequired": "3000000",
          "assetsRequiredUSDC": "3000000",
          "quotedByGVEPricePerAsset": "1000000"
        },
        {
          "assetId": "GLW",
          "assetsRequired": "7500000000000000000",
          "assetsRequiredUSDC": "3000000",
          "quotedByGVEPricePerAsset": "400000"
        }
      ],
      "rewardSplit": [
        {
          "streamId": "GLW_EMISSION",
          "splits": [
            {
              "walletAddress": "0xDelegatorA",
              "splitPercent6Decimals": "700000"
            },
            {
              "walletAddress": "0xDelegatorB",
              "splitPercent6Decimals": "300000"
            }
          ]
        },
        {
          "streamId": "USDG",
          "splits": [
            {
              "walletAddress": "0xInvestorA",
              "splitPercent6Decimals": "900000"
            },
            {
              "walletAddress": "0xInvestorB",
              "splitPercent6Decimals": "100000"
            }
          ]
        },
        {
          "streamId": "SGCTL",
          "splits": [
            {
              "walletAddress": "0xInvestorC",
              "splitPercent6Decimals": "1000000"
            }
          ]
        },
        {
          "streamId": "GLW",
          "splits": [
            {
              "walletAddress": "0xInvestorD",
              "splitPercent6Decimals": "1000000"
            }
          ]
        }
      ]
    }
  ]
}
```

In this example:
+ InvestorA provided the sGCTL deposit and gets 80% of rewards from the sGCTL virtual sub-farm
+ InvestorB provided the USDG deposit and gets 70% of rewards from the USDG virtual sub-farm
+ InvestorC provided the GLW deposit and gets 65% of rewards from the GLW virtual sub-farm
+ The operator (0xOperator) participates in all three virtual sub-farms but with different percentages

### Multi-Asset Processing Logic

The processing pipeline for multi-asset farms follows these behavioral requirements:

1. **Parse and Validate**: Validate the multi-asset farm input structure and field values

2. **Create Virtual Sub-Farms**: For each asset in the farm:
   + Calculate proportional impact: `impact = farm.netWeeklyImpactAssets × (asset.assetsRequiredUSDC / farm.totalProtocolDepositValue)`
   + Create a virtual sub-farm that competes in the competition identified by `(farm.regionId, asset.assetId)`
   + The virtual sub-farm inherits the `rewardSplit` configuration from its corresponding `AssetRequirement`
   + Each virtual sub-farm maintains independent progressive vault state

3. **Run Competition Simulator**: Process all virtual sub-farms through the standard competition logic
   + Each virtual sub-farm earns asset rewards based on performance
   + Each virtual sub-farm earns GLW inflation proportional to its deposits in its competition

4. **Apply Asset-Specific Reward Splits**: For each virtual sub-farm, distribute its rewards to wallet addresses
   + Use the reward split that was inherited from the asset deposit
   + Different virtual sub-farms from the same original farm may distribute to completely different wallet addresses

5. **Aggregate for Output**: Combine results from all virtual sub-farms back into multi-asset farm format for the output API

Note: The implementation must use virtual sub-farms to break down multi-asset farms into single-asset farms that are compatible with the original competition logic. After running the original implementation on these virtual sub-farms, the results are reconstructed back into the multi-asset farm format that matches the API output specification. This approach preserves the existing core competition logic without modification.

### Multi-Asset Output Format

The output format for multi-asset farms differs from single-asset farms to accommodate rewards earned in multiple asset types.

#### Farm Rewards

Each farm reward entry includes:

```json
{
  "id": "farm-0947b6e5-21dd-470a-b640-d7d319dd77b6-week-102",
  "farmId": "0947b6e5-21dd-470a-b640-d7d319dd77b6",
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
```

Key differences from single-asset output:
+ `id` is a composite identifier: `"{farmId}-week-{weekIndex}"`
+ `farmId` and `weekIndex` are provided separately for convenience
+ `assets` is an array containing earned amounts for each asset type
+ `protocolDeposit` is the total across all assets
+ `expectedProduction` is the farm's `netWeeklyImpactAssets`

#### Wallet Distributions

Wallet distributions can include multiple trace entries per farm (one per asset deposited):

```json
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
    }
  ]
}
```

### Multi-Asset Validation Rules

Additional validation rules for multi-asset farms:

+ Each farm must have at least one asset in the `assets` array
+ `assetId` must be one of: `"USDG"`, `"GLW"`, or `"SGCTL"`
+ Asset amounts must use correct scaling:
  + USDG: scaled by 1e6
  + GLW: scaled by 1e18
  + SGCTL: scaled by 1e6
+ `totalProtocolDepositValue` is the authoritative total (not validated against sum of `assetsRequiredUSDC`)
+ `quotedByGVEPricePerAsset` must be provided for each asset
+ All existing reward split validation rules apply:
  + Sum of `glowSplitPercent6Decimals` must equal 1000000
  + Sum of `depositSplitPercent6Decimals` must equal 1000000
  + Each wallet's split percentages apply uniformly to all asset types

### Backward Compatibility

The multi-asset functionality is provided through a new endpoint to maintain backward compatibility:

+ `/api/rewards-simulator` - continues to accept single-asset farms only (unchanged)
+ `/api/rewards-simulator-detailed` - continues to work with single-asset farms only (unchanged)
+ `/api/rewards-simulator-multi-asset` - new endpoint accepting multi-asset farms

This ensures existing integrations are not broken while enabling new multi-asset functionality.

## API Architecture

rewards-simulator offers an HTTP API that runs on
`localhost:35025/api/rewards-simulator`

The POST body is the input JSON object described above, and the response body
is the output JSON object described above.

An additional endpoint exists at `localhost:35025/api/rewards-simulator-detailed`
which returns the full internal state of the program. This means that the
return value has a list of competitions, and each competition has a list of
buckets, and each bucket has a list of farms, and the full suite of algorithmic
data structures are available in the output. This endpoint is usually used for
visualizations and for exploring the Glow solar rewards.

### Input Validation

Among other requirements mentioned elsewhere, the API checks that all numerical
values provided in the input are positive and non-zero. The one exception is
that netWeeklyImpactAssets is allowed to be zero.

### Error Behavior

Within the computation, several consistency checks verify that the internal
state matches the expectations of the theoretical algorithm. If one of the
consistency checks fails, the API will continue with the computation and will
produce the full set of output, and it will also return an error. Providing the
full output allows for the caller to get some insight into why things went
wrong.

## Rewards Competitions

There is one rewards competition per asset per region. This means that if there
are three regions, and three assets per region, then there are nine total
competitions. Rewards are computed independently for each competition.

## Algorithmic Architecture

The rewards-simulator is a pipeline with the following stages:

1. Parse the input from the user
2. Add any preload data (such as the v1 solar farms)
3. Run the competition simulator
4. Process and apply GCTL events
5. Apply the rewards splits and compose the output

Each step operates on the same set of core algorithmic data structures, which
get passed from step to step in the pipeline.

### Core Algorithmic Data Structures

```rs
pub struct RewardsState {
    pub competitions: HashMap<CompetitionID, Competition>,
    pub solar_farms: HashMap<String, SolarFarm>,
}

pub struct CompetitionID {
    pub region_id: u64,
    pub asset_id: String,
}

pub struct Competition {
    pub first_week: u64,
    pub final_week: u64,
    pub buckets: HashMap<u64, Bucket>,
}

pub struct Bucket {
    pub total_deposits: BigInt,
    pub total_impact_assets: BigInt,

    pub first_week_farms: Vec<String>,
    pub ongoing_farms: Vec<String>,
    pub last_week_farms: Vec<String>,

    pub farm_states: HashMap<String, FarmBucketState>,

    pub pool_net_assets: BigInt,
    pub pool_net_deposits: BigInt,

    pub glw_inflation: BigInt,
}

pub struct FarmBucketState {
    pub deposits_contributed: BigInt,
    pub impact_assets_contributed: BigInt,

    pub accumulated_drawdown: BigInt,
    pub net_overperformance: BigInt,
    pub rewards_this_week: BigInt,
}

pub struct RewardSplit {
    pub wallet_address: String,
    pub glow_split_percent_6_decimals: BigInt,
    pub deposit_split_percent_6_decimals: BigInt,
}

pub struct SolarFarm {
    pub farm_id: String,
    pub asset_id: String,
    pub region_id: u64,
    pub net_weekly_impact_assets: BigInt,
    pub protocol_deposit_value: BigInt,
    pub assets_required: BigInt,
    pub first_week: u64,
    pub weeks_alive: u64,
    pub reward_split: Vec<RewardSplit>,
}
```

## Preload Data

The rewards-simulator pipeline currently has the ability to preload one piece
of input, which is all of the rewards from Glow V1. If the query parameter
"preloadGlowV1=true" has been passed in, then the rewards-simulator will open
the file at 'v1-data.json' and merge it with the input provided by the user.

If the input provided by the user is empty, then the data inside 'v1-data.json'
will be used as the entire input.

The process for merging involves:

+ iterating over each element of 'cgpLeftovers' in 'v1-data.json' and adding
  that element to the user input. If there is no corresponding cgpLeftovers key
  in the user input, it will be created. If there is a corresponding cgpLeftovers
  key in the user input, then the two values will be added together. If there is
  no cgpLeftovers field at all in the user input, the field will be created.
+ Iterating over every solar farm in the solarFarms list and adding each farm
  to the user input. If a farm with an identical ID exists in the user input
  already, an error is returned.

If the 'preloadGlowV1=true' parameter has been set, the rest of the pipeline
will run with an expanded set of input which contains all of the v1 data.

The 'v1-data.json' file is not on-disk at runtime, rather it is embedded into
the binary at compile time. The data is embedded into the binary to ensure that
the user cannot trivially modify or misplace the data.

## The Competition Simulator

The competition simulator is responsible for figuring out how many rewards each
solar farm will earn from their impact asset production. This is the most
complex stage of the rewards-simulator pipeline.

### Competition Outline

Each competition is divided into buckets, one bucket per week. When a solar
farm is added to a competition, it divides its protocol deposit value evenly
between all of the buckets that the solar farm is participating in. For
example, solar farm 90 is competing in 60 weeks with a total protocol deposit
value of 6000, therefore it will contribute 100 protocol deposit value to each
of the buckets between week 96 and 155.

Each solar farm also contributes its weekly impact assets to each bucket.
Solar farm 90 would therefore contribute 1 impact asset to each bucket.

Each bucket, solar farms recover protocol deposit value based on the percentage
of impact assets that they contributed to that bucket. For example, a solar
farm that contributed 10% of the total impact assets to a bucket will receive
10% of the total protocol deposit value that was placed in the bucket.

As solar farms recover protocol deposit value, that protocol deposit value is
converted into asset rewards using the progressive vault model.

### The Progressive Vault Model

When a solar farm joins a competition, it distributes protocol deposit value to
each bucket, and then as it competes it recovers protocol deposit value. If a
solar farm outperforms, it will recover more protocol deposit value than it
distributed, which means it can collect assets from other solar farms. The
progressive vault model describes how assets are mapped to protocol deposit
value when rewards are collected.

Each solar farm has its own progressive vault, which has two stateful values
"accumulated drawdown" and "net overperformance".

Each competition has a shared resource called the performance pool, which has
two stateful values for the net protocol deposit value that has been added and
the net assets that have been added.

Each week, a solar farm will recover some amount of protocol deposit value.

If the solar farm overperforms, which means that it recovers more protocol
deposit value from the bucket than it distributed to the bucket, it will
increase its net overperformance value by the delta. For example, if a solar
farm contributed 100 protocol deposit value to a bucket and recovered 105
value, it will increase its net overperformance value by 5.

If a solar farm underperforms, which means that it recovers less protocol
deposit value from the bucket than it distributed to the bucket, it will
decrease its net overperformance by the delta. If this would cause the net
overperformance to go negative, it sets the net overperformance to 0 and adds
the difference to its accumulated drawdown.

For example, if a solar farm currently has a net overperformance of 3, and it
contributed 100 protocol deposit value to a bucket, and it recovered 95
protocol deposit value from that bucket, it will:

+ subtract 3 from its net overperformance, resetting it to 0
+ add 2 to its accumulated drawdown

If a solar farm underperforms, it must also make changes to the performance
pool. That extra 2 that was added to the accumulated drawdown must also be
added to the net protocol deposit value of the performance pool, and an amount
equal to `2 * farm.assets_required / farm.protocol_deposit_value` must be added
to the net assets of the performance pool.

In essence, because the underperformance of the solar farm caused the
accumulated drawdown to be increased by a value of 2, the farm had to send 2
protocol deposit value to the performance pool, as well as a proportional
amount of assets.

Each week, the amount of rewards that the solar farm collects is proportional
to the protocol deposit value that it recovers from the bucket. If the
accumulated drawdown of the solar farm is less than the original protocol
deposit value of the solar farm, then the solar farm will be able to collect
its own assets up until the accumulated drawdown of the solar farm is equal to
the original protocol deposit value of the solar farm. When the solar farm is
collecting its own assets, it must increase its accumulated drawdown by the
amount of protocol deposit value that it is collecting.

The amount of assets that the farm gets per unit of recovered protocol deposit
value is equal to `farm.assets_required / farm.protocol_deposit_value`.

If the accumulated drawdown of the farm is equal to the original protocol
deposit value of the farm, then the farm must collect its rewards from the
performance pool. For each unit of protocol deposit value that it collects from
the performance pool, it must decrease its net overperformance by a value of 1,
it must decrease the net protocol deposit value of the performance pool by one,
it must decrease the net assets of the performance pool by a value of
`pool.net_assets / pool.net_deposits`, and the farm will receive a number of
assets equal to `pool.net_assets / pool.net_deposits`.

This setup guarantees that after its final week, except for dust and rounding
errors, a solar farm will have an accumulated drawdown that is equal to its
original protocol deposit value, and it will have a net overperformance of
zero. These guarantees come from the fact that the solar farms are
participating in a zero-sum competition, therefore the total amount of
overperformance and underperformance is balanced.

### Building the Initial Data Structures

The first step of the competition algorithm is to fill out the basic
information for each competition and each bucket. This can be done with a
single pass over the farms. Before the pass starts, a `HashSet<String>` is
created which tracks the IDs of each farm that has been processed. An error is
returned if the input contains two farms with the same ID.

For each farm, we first check if the corresponding competition exists. All of
the competitions are tracked in a `HashMap<CompetitionID, Competition>`, so we
can create the competition ID that corresponds to the competition identified by
the farm, and then create a new competition if one doesn't exist.

If we are creating a new competition, the first week of the competition will be
set equal to the first week of this farm, and the last week of the competition
will be set equal to `farm.first_week + farm.weeks_alive - 1`. Both the
`first_week` and `weeks_alive` values on all farms must be positive non-zero
values that are less than 2^12, and `weeks_alive` must be greater than or equal
to 2.

One bucket will be created per week that the farm is participating in the
competition, and the `total_deposits` and `total_impact_assets` values for
each bucket will be set. The `total_deposits` for each bucket will be set equal
to `farm.protocol_deposit_value / farm.weeks_alive` and the
`total_impact_assets` for each bucket will be set equal to
`farm.net_weekly_impact_assets`.

The farm then has to add itself to the appropriate vec in the bucket. If this
is the first bucket where the farm appears, it adds itself to
`first_week_farms`. If this is the last bucket where the farm appears, it adds
itself to `last_week_farms`, otherwise it adds itself to `ongoing_farms`.

The farm then creates a `FarmBucketState` for itself and adds it to the
`farm_states` field in the bucket. The `deposits_contributed` value is set to
`farm.protocol_deposit_value / farm.weeks_alive` and the
`impact_assets_contributed` value is set to `farm.net_weekly_impact_assets`. The
accumulated drawdown and net overperformance values are both set to zero -
those will be computed dynamically later.

If a farm is being added to an existing competition, most of the steps are the
same, except that it will be expanding the `first_week` and `final_week` values
as necessary, creating the corresponding new buckets if necessary, and then
inserting itself into any buckets that already exist. When a farm inserts
itself into an existing bucket, it increments `total_deposits` and
`total_impact_assets` by the appropriate amount, appends itself to the
appropriate list of farms for that bucket, and then creates a `FarmBucketState`
for itself.

After each farm has been processed, there should be a set of competitions, each
competition should have a set of buckets, and each bucket should have a set of
farms.

### Dynamically Computing Competition Rewards

Up until this point, all of the `pool_net_deposits` and `pool_net_assets` and
`accumulated_drawdown` and `net_overperformance` and `rewards_this_week` values
have been initialized to 0. The algorithm will go competition-by-competition,
and bucket-by-bucket, and compute the intermediate values for each farm, which
will ultimately compute the `rewards_this_week` values for each farm.

The loop structure is roughly:

```
for each competition,
    for each bucket,
        for each farm,
            run some code,
```

The competitions can be processed in any order. The buckets must be processed
in numerical order. It's possible that a competition has gaps, if there were
farms added to the competition with non-overlapping week ranges. This means
that the `first_week` might be 100, the `final_week` might be 500, and there
are no buckets for weeks 250-300. In that case, the missing buckets are simply
skipped.

When the algorithm progresses to the next bucket in a competition, it needs to
grab the `pool_net_deposits` and `pool_net_assets` values from the previous
bucket. If the immediately previous bucket does not exist, it will leave the
values set to 0. It does not need to search backwards beyond the immediately
previous bucket.

When the algorithm processes the next farm in a bucket, it will need to grab
the `accumulated_drawdown` and `net_overperformance` values for the farm in the
previous bucket. If the farm did not appear in the previous bucket, then these
values will remain set to 0.

Then, each farm follows the steps provided in the progressive vault model
description. First the farm computes how much protocol deposit value it
recovers:

```
deposits_recovered = bucketState.impact_assets_contributed * bucket.total_deposits / bucket.total_impact_assets
```

If `deposits_recovered` is larger than the `bucketState.deposits_contributed`,
then the delta needs to be added to `bucketState.net_overperformance`.

If `deposits_recovered` is smaller than the `bucketState.deposits_contributed`,
then the delta needs to be subtracted from `bucketState.net_overperformance`.

If that subtraction would underflow `bucketState.net_overperformance`, then
`bucketState.net_overperformance` is set to 0 and the remaining amount that
needs to be subtracted gets added to `bucketState.accumulated_drawdown`. The
amount that gets added to `bucketState.accumulated_drawdown` is called the
`penalty_drawdown`.

If the `penalty_drawdown` is greater than zero, then it needs to be added to
`bucket.pool_net_deposits`. The penalty assets also need to be added to
`bucket.pool_net_assets`. The penalty assets can be computed with:

```
penalty_assets = penalty_drawdown * farm.assets_required / farm.protocol_deposit_value
```

After this step is complete, the `rewards_this_week` value must be computed. It
is computed by taking the `deposits_recovered` and converting them into assets.
As long as `bucketState.accumulated_drawdown` is less than
`farm.protocol_deposit_value`, the farm will collect assets from its own vault.
And if the farm has fully drawn down its vault, then it will collect assets
from the pool.

When the farm collects assets from its own vault, the number of assets that it
gets per deposit recovered is equal to `farm.assets_required /
farm.protocol_deposit_value`. As the farm collects assets from its own vault,
it needs to increment the `bucketState.accumulated_drawdown` value. As soon as
the `bucketState.accumulated_drawdown` value is equal to
`farm.protocol_deposit_value`, the farm needs to switch to collecting assets
from the pool.

When collecting assets from the pool, the number of assets the farm gets per
deposit recovered is equal to `bucket.pool_net_assets /
bucket.pool_net_deposits`. As the farm collects assets from the pool, it needs
to decrease its own `bucketState.net_overperformance` and it also needs to
decrease both `bucket.pool_net_deposits` and also `bucket.pool_net_assets`.

The final value for the `bucketState.rewards_this_week` field is equal to the
total amount of assets that were collected from both the farm's own vault as
well as the pool. In most cases, all of the assets will come from either one or
the other, but sometimes a farm will collect from both.

The farm updates for a bucket are actually done in two passes. The first pass
determines how many assets each farm is contributing to the pool, and the
second pass determines how many assets each farm is withdrawing from the pool.
This ensures that the farms will have the same outcome independent of what
order they are processed in.

If this is the final bucket for the farm, when all computations are done a
consistency check should be run to make sure that
`bucketState.accumulated_drawdown` is nearly equal to
`farm.protocol_deposit_value`, and also that `bucketState.net_overperformance`
is close to zero. These values might be slightly off due to dust that was
discarded.

If this is the final farm for the final bucket in a competition, or if the next
bucket in the competition does not exist, then a consistency check should be
performed to verify that `bucket.pool_net_assets` and
`bucket.pool_net_deposits` are both zero. If the values are close to zero, that
is okay as well, because the algorithm does round down in places which discards
dust. This could cause some of the values to not perfectly reach zero, and that
is okay.

### Special Case: CGP Leftovers

For only the competition in the cgp region (region 1) with the "USDG" asset,
farms will get bonus rewards for weeks where there are `cgpLeftovers`. For each
protocol deposit value that the farm recovers, it can add
`cgpLeftovers[weekNum] / bucket.total_deposits` to its `rewards_this_week`.
This addition does not have any interaction with the other variables - it won't
modify `net_overperformance` or `accumulated_drawdown` or change any of the
pool state, it just directly increases the `rewards_this_week` value for each
farm proportional to the deposits that the farm recovered.

## Process and Apply the GCTL Events

There is currently an optional input field called "gctlDistribution". If that
field is not provided, the GLW distribution will default to 120,641 GLW tokens
for region 1, 18,119 GLW tokens for region 2, 18,119 GLW tokens for region 3,
and 18,119 GLW tokens for region 4. These values will need to be scaled by
1e18.

If the "gctlDistribution" field is provided, the GLW distribution for a region
will be set equal to 175,000 GLW tokens multiplied by that region's percent of
the total GCTL.

For example, if a region has 100 GCTL tokens staked to it, and there are 1750
GCTL tokens total staked across all regions, then that region will receive
10,000 GLW per week. The GLW values will need to be scaled by 1e18. If a region
has 0 GCTL tokens staked to it, it will have 0 GLW inflation.

To apply the `glw_inflation` to buckets, the algorithm will first determine the
range of weeks that need to be checked. It does this by iterating over every
competition and taking the lowest `first_week` value and the highest
`final_week` value and using those to put bounds on all the weeks that must be
checked.

Then, the algorithm must figure out how many GLW tokens are to be distributed
to each competition. This depends on two factors: first it depends on how many
GLW tokens are allocated to the competition's region for this week, and second
it depends on how many `total_deposits` there are for each competition inside
this region for this week.

All of the competitions within a week are sharing the region's GLW rewards.
Therefore, if a region has 10,000 GLW tokens, the sum of all the
`glw_inflation` values for each week across all of the region's competitions
must be 10,000 GLW.

The amount of that 10,000 GLW that is distributed to each competition is
proportional to the `total_deposits` of each competition. So if there are two
competitions for the region in a given week, and one of the competitions has a
`total_deposits` of "100" and the other has a `total_deposits` of "900", then
the algorithm will set the `glw_inflation` of the first competition to 1,000
GLW and it will set the `glw_inflation` of the second competition to 9,000 GLW.

This means that if a region only has one competition in a given week, the
`glw_inflation` of the competition for that week will be equal to the GLW that
is being distributed to the region.

If a region has zero competitions in a week, the distribution for that region
is skipped entirely for that week. The GLW tokens are not redistributed to
other regions. If a region has only one competition, that competition will
receive all of the GLW tokens for that region that week.

## Applying the Rewards Splits and Creating the Final Output

After the algorithm has been run, there will be a bunch of competitions, each
with a bunch of buckets, and each bucket will have a bunch of farms, and each
farm will have a `rewards_this_week` value. Each farm will also have a list of
rewards splits, which will now need to be put into the final output.

The final output itself is designed to integrate with a different system,
therefore the composition of the output is a relatively significant departure
from the internals of the rewards simulator.

The output itself is a map from week number to a distribution object. That
distribution object is broken into walletDistributions, farmRewards,
regionData, and warnings. The warnings field is used to present any errors or
consistency check problems that occurred during execution. Every warning that
gets produced should be appended to the list of warnings for every week.

The regionData is a quick summary of the total protocol deposits and carbon
credit production that happened in each competition, separated by region.

There is some redundant information between the walletDistributions and the
farmRewards. The farmRewards essentially state how each farm has received
rewards, and the walletDistributions state how those rewards get applied to
wallets based on the reward splits.

The farmRewards are the closest thing to the algorithmic internals of the
program, except that they are all rolled up into a single array rather than
being separated by competition. The farmRewards can be built for a week by
iterating over every competition, determining which competitions are active
that week, and then iterating over every farm in the bucket for that week and
composing the array element for that farm.

The wallet distributions can be built in a similar way, and can be built as the
farmRewards are being built. When the farmRewards element is being created for
a farm, the algorithm can iterate over all of the rewards splits for the farm
and add them to the walletDistributions object. The
`glow_split_percent_6_decimals` field says what percentage of the inflation
rewards go to that wallet address. And similar for the deposit split percents.
The number is out of 1000000, so 50000 means the wallet is getting 5%.

There is a key constraint however. Each "userAddress" may only appear in the
walletDistributions one time. If a certain wallet address has already been
added to the walletDistributions object from another rewards split (either in
the same farm or a different farm), then the new rewards split must be merged
into the existing walletDistributions element for that address.

The merging process is simple. You sum together the glowInflationEarned fields,
you append a new element to the "traces" array, and you merge together the
"assetsEarned" map such that if the new trace is a new asset, that asset gets
inserted into the map, and if the asset is already in the map then the new
value is summed into the existing value.

When the whole process is done, the output object should be ready, and can be
returned out the API.

The API features a query parameter "week" which allows the caller to specify
that they only want the data for a single week. If this flag is set, the return
value will just be the object for that week.

Before the final output is provided to the caller, some consistency checks
should be performed. First, it should be verified that for each
walletDistribution object, the "assetsEarned" value for each currency is equal
to the sum of all the "amount" values for that currency in the corresponding
"traces" array. Second, it should be verified that the "glowInflationEarned"
for a wallet distribution matches the sum total of all "glowInflationReward"
values for all of its traces. Third, it should be verified that the sum total
of all "glowInflationEarned" across all walletDistributions matches the sum
total of all "glowInflationEarned" across all farmRewards objects. Finally, it
should be verified that the sum total of all "assetsEarned" across all
walletDistributions object should match the sum total of all "assetsEarned"
across all farmRewards objects for each type of asset.

## Coding Conventions

### Naming

It should be noted that `camelCase` is used for all JSON variables, including
all variables across all endpoints that are used as input or output in the API.
All of the internal rust variables use `snake_case`, as is idiomatic for rust.

### Precision and Rounding

Because all computations must be deterministic and precise, BigInts are used
everywhere. In practice, input values will have a scaling factor of between
1e6 and 1e18. For example, 1 usdg will be passed into the input as 1000000
usdg. This means that when dividing that 1 usdg into 100 buckets, there is
ample precision to ensure a nearly lossless distribution. Because all of the
scaling is handled by the input values, no internal scaling is needed.

Anything that cannot be distributed evenly is considered to be "dust" and
should be discarded. When distributing, it is important to always round down,
thus ensuring that the total amount of assets that are distributed as rewards
never exceeds the total amount of assets that were provided as inputs.

During consistency checks, the tolerance for precision errors should be set
equal to 1 unscaled unit. For example, the tolerance when checking usdg should
be 1e6, and the tolerance when checking impactAssets should be 1e18.

Use of floating points is not allowed.

### BigInt JSON Encoding

BigInt values are always encoded as JSON strings for both inputs and outputs
across all endpoints.

## Rewards Visualizer

The rewards visualizer is a visualizer that is served by the server at
index.html. The source code for the visualizer is stored at `/src/web/`

The visualizer is implemented in pure javascript/html/css - there are no
dependencies, including no dependencies on node or typescript.

The visualizer is a frontend that allows the user to design a set of rewards
competitions (including using preloads), then run the rewards simulation, then
visually explore all of the output. The default competition is a competition
for the "simulation" region that uses the "GLW" asset, but multiple
competitions are supported.

The page is split horizontally. The top portion of the page contains the "input
designer", which allows the user to add competitions and also add solar farms
to each competition, and the bottom portion of the page contains the output
visualization. The output visualization has two views that the user can switch
between, one view is a per-week visualization, and one view is a per-farm
visualization.

In the output visualization, the user can switch between competitions, showing
one competition at a time.

### The Input Designer

The input designer allows the user to select a competition, and then view and
modify the farms in that competition. Clicking on the 'Add Competition' button
will open a modal that allows the user to configure the `regionId` and
`assetId` for the competition.

This readme uses $ASSET to indicate a place where the asset ticker should be
used, and the asset ticker is always an all-caps version of the assetId. For
example, if the assetId is "GLW", then $ASSET is GLW.

Each farm is its own visual card, and the user can configure the following
values for the farm:

+ The first week that the farm joins the competition
+ The number of weeks the farm is in the competition
+ The number of impact assets the farm produces each week
+ The protocol deposit of the farm (denominated in dollars)
+ The GLW token price (denominated in dollars)

When the page loads, it autogenerates a competition with the regionId
"simulation" and assetId "GLW" which shows 3 farms:

+ The joining weeks for the farms are week 1, week 2, and week 2 respectively
+ The weeks alive for the farms are 5 for each farm
+ the weekly CC are 0.08, 0.1, and 0.12 respectively
+ The deposits for the farms are $40,000, $80,000, and $50,000 respectively
+ The $ASSET price for the farms is $0.30, $0.40, and $0.40 respectively

There is also a card that allows the user to add a new farm. Clicking that card
will will open a form that the user can fill out with all the fields. The user
can also assign the farm ID. The default values of the form match the values
for the first farm, except that the ID is set using a counter (because
duplicate IDs are not allowed). Upon clicking 'submit', the new farm is
inserted at the end.

There is a button in the input designer that allows the user to sort the farms,
if clicked the farms will be sorted by their first week. If two farms have the
same first week, they will be sorted by their farm ID.

Every farm has both an edit button, which will allow the user to edit the
fields of the farm, and a delete button, which will allow the user to delete
the farm.

When converting this into input that is sent to the rewards-simulator-detailed
endpoint, the `netWeeklyImpactAssets` is scaled up by a factor of 1e18 from
what the user inputs, the protocol deposit value is scaled up by a factor of
1e6 from the user inputs, the `assetsRequired` is set equal to the user-set
protocol deposit divided by the user-set asset price and then scaled up by a
factor of 1e18, and the `firstWeek` and `weeksAlive` are set to the values
provided by the user.

If the asset is 'usdg', then 'assetsRequired' needs to be scaled by a factor of
1e6 instead of 1e18. This special case must be handled carefully, especially
when importing rewards from V1.

The user interface accepts floating point inputs from the user for all values
that are going to be scaled up when they are submitted to the API endpoint.

An 'import v1 farms' button exists at the bottom which allows the user to
toggle importing the v1 farms. If the user does enable this toggle, the
visualization will tell the API to load the v1 farms, which means that more
competitions may be added and a bunch of solar farms will be added to the final
output.

A 'simulate rewards' button exists at the bottom of the input designer which
will send all of the farms to the rewards-simulator-detailed endpoint and then
parse the response and present the rewards visualization.

### The Output Visualization

The output visualization shows each competition that appeared in the output,
allowing the user to switch between competitions. When viewing the outputs for
a specific competition, the user has the ability to look at the per-week
visualization or the per-farm visualization.

### The Per-Week Visualization

The per-week visualization shows all of the weeks that were simulated. The top
of the visualization is a neatly compacted list of every week, where each week
only shows the the number of farms in that week. It must be emphasized that
this view must be compact, ideally more than a dozen weeks can fit on each row,
and dozens of rows can fit on each page.

When a user clicks on a week, a detailed view for the week is shown below the
compact list of weeks. This detailed view shows all of the key week details,
and it also shows all of the farms that are participating in that week, one
card per farm. By default, the first week is active.

The detailed overview for the week displays:

+ the total deposits for that week
+ the total impact assets for that week
+ the GLW inflation for that week
+ the net assets in the pool

All numbers are presented to the user as scaled down floating point values,
with a sensible amount of precision.

Each farm card displays the following information:

+ The number of GLW tokens earned this week (gently highlighted)
+ The rewards for the farm that week (denominated in $ASSET) (gently highlighted)
+ The deposits contributed by the farm to that week
+ The impact assets contributed by the farm to that week
+ The number of $ASSET rewards recovered from the farm's own vault
+ The number of $ASSET rewards recovered from the pool
+ The deposits recovered by the farm in that week (denominated in dollars)
+ The number of weeks remaining before the farm is no longer active
+ The accumulated drawdown of the farm as of that week
+ The net overperformance of the farm as of that week

The deposits recovered will need to be calculated by the frontend using the
equation `total_deposits * impact_assets_contributed / total_impact_assets`

The number of $ASSET rewards recovered from the pool can be calculated with the
following rough strategy:

```
base_overperformance = 0
if deposits_recovered > deposits_contributed {
  base_overperformance += deposits_recovered
  base_overperformance -= deposits_contributed
}
base_overperformance += prev_week_net_overperformance
base_overperformance -= net_overperformance
glw_from_pool = base_overperformance * pool_net_assets / pool_net_deposits
```

The number of $ASSET rewards recovered from the farm's own vault is equal to the
total rewards minus the glw recovered from the pool.

The number of GLW tokens earned this week is equal to `glw_inflation *
deposits_contributed / total_deposits`

### The Per-Farm Visualization

The per-farm visualization shows all of the farms that were simulated. The top
of the visualization is a neatly compacted list of every farm, where each farm
only shows its total deposit. It must be emphasized that this view must be
compact, ideally more than a dozen farms can fit on each row, and dozens of
rows can fit on each page.

When a user clicks on a farm, a detailed view for that farm is shown below the
list of farms. The detailed view starts with a centered overview of the farm,
which contains the following details:

+ The total deposit for the farm
+ The assets required for the farm
+ The total GLW inflation for the farm across all weeks
+ The total asset rewards across all weeks (the sum of all `rewards_this_week` values)

Below the overview of the farm is a one card for each week. Each card shows:

+ The number of GLW inflation earned by the farm that week (gently highlighted)
+ The rewards for the farm that week (denominated in $ASSET) (gently highlighted)
+ The deposits contributed by the farm to that week
+ The impact assets contributed by the farm to that week
+ The deposits recovered by the farm in that week (denominated in dollars)
+ the total impact assets for that week
+ the total deposits for that week
+ The total GLW inflation distributed to the competition that week
+ the net assets in the pool for that week
+ the net deposits in the pool for that week
+ The number of $ASSET rewards recovered from the farm's own vault
+ The number of $ASSET rewards recovered from the pool
+ The accumulated drawdown of the farm as of that week
+ The net overperformance of the farm as of that week

The deposits recovered will need to be calculated by the frontend using the
equation `total_deposits * impact_assets_contributed / total_impact_assets`

The number of $ASSET rewards recovered from the pool can be calculated with the
following rough strategy:

```
base_overperformance = 0
if deposits_recovered > deposits_contributed {
  base_overperformance += deposits_recovered
  base_overperformance -= deposits_contributed
}
base_overperformance += prev_week_net_overperformance
base_overperformance -= net_overperformance
glw_from_pool = base_overperformance * pool_net_assets / pool_net_deposits
```

The number of $ASSET rewards recovered from the farm's own vault is equal to the
total rewards minus the glw recovered from the pool.

The number of GLW tokens earned by the farm in a week is equal to
`glw_inflation * deposits_contributed / total_deposits`

Clicking on another farm will update the view to show the details and week
cards for that farm.

### Displaying Numbers

For all numbers that are strictly less than 1,000, the UI should display the
number with 2 decimals of precision. For all numbers that are larger than or
equal to 1,000 and less than 1.00m, the numbers should be displayed with commas
and there should be 0 decimals of precision. For numbers equal to or greater
than 1.00m and less than 1.00e15, they should be displayed with 2 full decimals
of precision (even if there are trailing zeroes) and be followed by 'm' or 'b'
or 't' depending on the number size. For numbers greater than or equal to
1.00e15, they should be displayed using engineering notation with two decimals
of precision. For example, 656.92e18.

Engineering notation here means scientific notation, but the orders of
magnitude are always divisible by three. E.g. 1e15, 10e15, 100e15, 1e18.

### Pagination

Both the per-week and the per-farm views can end up with long lists. When the
simulator is porting data from the real world, there can be over 100 weeks
listed and over 100 solar farms listed. Therefore, the output needs to be
paginated.

### Testing

The competition visualizer is tested using headless chromium. The webapp itself
features a test harness, and the index.html page will load the test harness and
test code if the query parameter `?test=1` or `?runTests=true` is provided.

The file `tests.js` can be used to inspect the DOM and manipulate the webpage
headlessly as a user would, checking that everything seems to be in order after
key actions are taken.

## Glow Branding Guidelines

This is a Glow project, which means that it needs to adhere to the Glow
branding guidelines. Any frontend, including the visualizer, must adhere to the
brand guidelines for design, use of fonts, etc.

Glow’s brand identity fosters authentic credibility, setting the visual
foundation for a global climate-impact leadership enterprise.

### Color Palette

Our color palette is designed to reinforce and support the Recursive Gradient,
playing a functional yet pivotal role across brand and digital. Note that the
Orange is an accent colour only, primarily for use in UI contexts as shown
below.

+ #050505 - Black
+ #FAFAFA - Light Grey
+ #F3F3F3 - Medium Grey
+ #FFFFFF - White
+ #FFB472 - Orange (accents only)

### Symbol, Wordmark, and Lock-up

The lock-up is the combination of the Glow symbol and wordmark. The Glow symbol
represents the recursive power of solar technology. It is built from the
silhouettes of solar panels, rotating in a clockwise formation to form a sun,
and radiating outwards.

The Glow wordmark is represented with the following SVG:

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2500 824.89" fill="currentColor">
  <path d="M380.37,482.5h256.84v184.12c-25.41,17.28-57.97,31.84-97.59,43.78-39.62,11.96-82.61,17.93-128.96,17.93-58.41,0-109.76-12.49-154.25-37.58-44.52-24.97-79.15-61.48-103.79-109.38-24.64-47.91-37.02-104.06-37.02-168.37s11.52-120.48,34.74-168.37c23.24-47.91,55.37-84.73,96.52-110.58,41.15-25.74,87.51-38.67,139.06-38.67,68.72,0,122.02,14.77,159.8,44.32,37.78,29.55,62.33,72.33,73.5,128.49h118.86c-6.73-50.94-24.32-96.56-52.76-136.97-28.45-40.39-67.3-72.33-116.7-95.91C519.33,11.73,460.6,0,392.53,0c-77.71,0-146.34,17.6-205.81,52.79-59.48,35.09-105.41,83.97-137.97,146.42C16.28,261.68,0,332.15,0,412.98s16.4,152.3,49.4,214.43c32.89,62.13,80.66,110.57,143.5,145.34l.02.02.21-.11c62.73,34.77,137.21,52.14,223.18,52.14,58.29,0,118.33-10.01,179.99-29.77,61.76-19.77,109.74-43.9,144.15-72.35v-334.45h-360.08v94.28Z"/>
  <path d="M1516.98,272.74c-44.52-24.67-94.79-37.05-150.91-37.05s-106.39,12.38-150.89,37.05c-44.52,24.65-79.47,59.53-104.88,104.38-25.41,44.87-38.11,95.81-38.11,152.62s12.7,107.77,38.11,152.62c25.41,44.87,60.36,79.85,104.88,104.94,44.5,25.09,94.76,37.58,150.89,37.58s106.39-12.49,150.91-37.58c44.5-25.11,79.47-60.08,104.86-104.94,25.41-44.87,38.11-95.81,38.11-152.62s-12.7-107.77-38.11-152.62c-25.39-44.85-60.36-79.72-104.86-104.38ZM1529.25,635.77c-15.31,31.82-37.02,56.7-65.03,74.63h.02c-28.01,17.93-60.8,26.94-98.14,26.94s-70.12-8.78-98.13-26.39c-28.01-17.58-49.72-42.25-65.03-74.07-15.31-31.84-23.02-67.89-23.02-108.31s7.5-76.35,22.48-107.75c14.96-31.51,36.46-55.72,64.47-73,28.01-17.28,61.1-25.85,99.21-25.85s71.2,8.78,99.21,26.39c28.01,17.6,49.51,42.14,64.49,73.54,14.96,31.49,22.46,67.01,22.46,106.66s-7.71,75.39-23.02,107.22Z"/>
  <polygon points="2399.05 252.55 2266.71 725.06 2152.29 252.55 2035.59 252.55 1926.81 725.06 1793.3 252.55 1688.99 252.55 1688.97 252.55 1688.88 252.55 1854.86 808.16 1991.74 808.16 2094.99 367.04 2198.23 808.16 2332.83 808.16 2500 252.55 2399.05 252.55"/>
  <polygon points="785.22 121.18 862.37 121.18 862.37 808.16 966.7 808.16 966.7 16.84 785.22 16.84 785.22 121.18"/>
</svg>

The Glow symbol is represented with the following SVG:

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 237 239" fill="currentColor">
  <path d="M75.3805 0L63.6266 59.9862L102.965 86.2833L114.719 26.2971L75.3805 0Z"/>
  <path d="M172.625 6.85553L121.898 40.9609L130.935 86.4428L181.663 52.3399L172.625 6.85553Z"/>
  <path d="M236.181 80.1185L176.197 68.3622L150.952 106.128L210.935 117.882L236.181 80.1185Z"/>
  <path d="M228.455 176.248L194.35 125.521L150.722 134.189L184.828 184.916L228.455 176.248Z"/>
  <path d="M131.043 153.862L119.29 213.849L155.481 238.043L167.235 178.056L131.043 153.862Z"/>
  <path d="M102.84 154.032L52.1126 188.138L60.4102 229.908L111.14 195.803L102.84 154.032Z"/>
  <path d="M83.1263 134.28L23.1426 122.524L0 157.142L59.9862 168.896L83.1263 134.28Z"/>
  <path d="M9.2948 63.4374L43.4002 114.165L83.3131 106.235L49.2101 55.5074L9.2948 63.4374Z"/>
</svg>

### Recursive Gradient

Our Recursive Gradient is one of our primary brand elements, creating instantly
recognizable and uniquely Glow brand moments. In smaller applications such as
social icons, please default to using predefined crops of the gradient (A/B/C).

Here are the gradients defined in CSS:

.glow-gradient {
  background: linear-gradient(111.06deg, #f7fcc4 12.01%, #ccffd4 39.47%, #dcc4ff 93.61%);
}

.glow-gradient-a {
  background: linear-gradient(111.06deg, #ccffd4 12.01%, #dcc4ff 93.61%);
}

.glow-gradient-b {
  background: linear-gradient(111.06deg, #f7fcc4 12.01%, #ccffd4 93.61%);
}

.glow-gradient-c {
  background: linear-gradient(111.06deg, #dcc4ff 12.01%, #f7fcc4 93.61%);
}

### Fonts

Söhne is our primary brand type family. It is a contemporary yet timeless
sans-serif with clean forms and commanding presence.

Duplicate Slab is our secondary type family. It is a humanistic yet grounded
serif, boasting a natural look offset by a strong personality.

### Font Files

The font files are available in the following locations:

+ src/web/assets/DuplicateSlab-Regular.otf
+ src/web/assets/Söhne-Buch.otf
+ src/web/assets/Söhne-Halbfett.otf
+ src/web/assets/Söhne-Leicht.otf

Though the fonts cannot be included (for licensing reasons) in the repo itself,
it is safe to assume that whoever is running the binary has put the correct
font files in the asset folder. These fonts are properly licensed and may be
used with this project.
