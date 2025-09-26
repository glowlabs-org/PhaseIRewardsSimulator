# User Specification

rewards-simulator is a rust program that takes as input an array of solar farms
and produces as output a list of rewards that each solar farm would receive on
Glow V2 Phase I for each week.

## Input and Output Structure

The input to this program is a json array of farms that defines all of the
farms that are enrolled in the rewards program. The input defines what week the
farm joins the rewards program, and how many weeks the farm is participating in
the rewards program. The farm needs to participate in the rewards program for
an integer number of weeks, but it can be any number of weeks, which allows for
farms that are being ported from V1 to V2 to define a shorter tenure.

The asset id defines which asset is being used for the protocol deposit and
rewards, and the `assets_required` field defines how many assets are
participating in the weeks that remain. Solar farms that are being ported from
V1 to V2 therefore should not state their whole protocol deposit, but instead
should state the amount of protocol deposit that was remaining in the V1
buckets before the solar farm was ported over.

A special field called `cgp_leftovers` is provided which defines the total
amount of residual early liquidity rewards that were remaining in the V1
buckets.

```json
{
  "cgp_leftovers": [
    "96": 235,
    "97": 367,
  ],
  "solar_farms": [
    {
      "farm_id": "45",
      "asset_id": "glw",
      "region_id": "cgp",
      "weekly_carbon_credits": 1,
      "protocol_deposit_value": 10000,
      "assets_required": 25000,
      "rewards_address": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
      "first_week": 96,
      "weeks_alive": 100
    },
    {
      "farm_id": "90",
      "asset_id": "usdg",
      "region_id": "utah",
      "weekly_carbon_credits": 1,
      "protocol_deposit_value": 6000,
      "assets_required": 6000,
      "rewards_address": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
      "first_week": 96,
      "weeks_alive": 60
    }
  ]
}
```

Note: the `farm_id` is a string so that fractionalized farms can be represented
with suffixes. For example `"45_frac_1"`.

Note: the `rewards_address` must be a valid ethereum mainnet address.

Note: the 'cgp leftovers' is a map from week number to the amount of usdg that
was put into the corresponding bucket by the early liquidity contract. This map
is used to distribute bonus rewards to solar farms participating in the cgp
region with the usdg asset.

The output will be a json object that contains all of the rewards that will be
distributed to each solar farm in each week:

```json
{
  "total_regions": 2,
  "regional_stats": [
    {
      "region": "cgp",
      "assets": ["glw"]
    },
    {
      "region": "utah",
      "assets": ["usdg"]
    }
  ],
  "weekly_rewards": [
    {
      "week_number": 96,
      "per_farm_rewards": [
        {
          "farm_id": "45",
          "asset_id": "glw",
          "region_id": "cgp",
          "amount": 250,
          "rewards_address": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D"
        },
        {
          "farm_id": "90",
          "asset_id": "usdg",
          "region_id": "utah",
          "amount": 100,
          "rewards_address": "0xa273164a466dbF9F0173996078fb382acC73F9E3"
        }
      ]
    }
  ]
}
```

## API Architecture

rewards-simulator offers an http API that runs on
localhost:35025/api/rewards-simulator

The post body is the input json object described above, and the response body
is the output json object described above.

## Porting Solar Farms From V1 to V2

The solar farms that are provided as inputs to the rewards-simulator are a mix
of Glow solar farms that enrolled after V2 was launched, and solar farms that
enrolled before V2 was launched. Because V1 used a moderately different rewards
system, the V1 farms need to be ported into V2. The rewards-simulator does not
provide any logic to do the porting, but here is an explanation for how the
porting process should work:

When a solar farm joined Glow V1, it provided a protocol deposit. That deposit
was split up into 192 equal pieces and each piece was placed into a bucket. The
first 16 buckets didn't receive any pieces, and the next 192 buckets received
one piece each.

When a solar farm transitions from V1 to V2, it will "reclaim" the protocol
deposit that it placed into each bucket which has not yet distributed rewards.
The total sum of all pieces that are reclaimed is the `protocol_deposit_value`
that is provided as input to the rewards-simulator.

The number of buckets, including the first 16 empty buckets, that the solar
farm has placed rewards in which have not yet distributed rewards determine the
solar farm's `v1_lifespan`. For example, if a solar farm joined on week 50, it
will have placed rewards in buckets 50-257. If the V2 transition happens on
week 58, then the solar farm's `v1_lifespan` is 200 weeks.

To convert `v1_lifespan` to `weeks_alive` for the rewards simulator, you use
the function `FLOOR(v1_lifespan / 2.08)+1`. Be sure to use floating points for
the intermediate computation.

After all of the solar farms have been ported from V1 to V2, some amount of
money will be left in the buckets from the early liquidity. There will also be
money left in the buckets from farms that got banned from Glow V1 for fraud.
The total amount of leftover money in each bucket is provided in the
`cgp_leftovers` map.

There is a special case where one farm potentially needs to be refunded. To
handle that special case, you can subtract a proportional amount from each week
of leftovers. It's an imprecise fudge, but it gets the job done.

## Rewards Competitions

There is one rewards competition per asset per region. This means that if there
are three regions, and three assets per region, then there are nine total
competitions. Rewards are computed independently for each competition.

## Creating Competition Buckets

Each competition is divided into buckets, one bucket per week. When a solar
farm is added to a competition, it divides its protocol deposit value evenly
between all of the buckets that the solar farm is participating in. For
example, solar farm 90 is competing in 60 weeks with a total protocol deposit
value of 6000, therefore it will contribute 100 protocol deposit value to each
of the buckets between week 96 and 155.

Each solar farm also contributes its weekly carbon credits to each bucket.
Solar farm 90 would therefore contribute 1 carbon credit to each bucket.

Each bucket, solar farms recover protocol deposit value based on the percentage
of carbon credits that they contributed to that bucket. For example, a solar
farm that contributed 10% of the total carbon credits to a bucket will receive
10% of the total protocol deposit value that was placed in the bucket.

As solar farms recover protocol deposit value, that protocol deposit value is
converted into asset rewards using the progressive vault model.

## The Progressive Vault Model

When a solar farm joins a competition, it distributes protocol deposit value to
each bucket, and then as it competes it recovers protocol deposit value. If a
solar farm outperforms, it will recover more protocol deposit value than it
distributed, which means it can collect assets from other solar farms. The
progressive vault model describes how assets are mapped to protocol deposit
value when rewards are collected.

Each solar farm has its own progressive vault, which has two stateful values
"accumulated drawdown" and "net overperformance".

Each competition has a shared resource called the performance pool, which has
two stateful values "net protocol deposit value" and "net assets".

Each week, a solar farm will recover some amount of protocol deposit value.

If the solar farm overperforms, which means that it recovers more protocol
deposit value from the bucket than it distributed to the bucket, it will
increase its "net overperformance" value by the delta. For example, if a solar
farm contributed 100 protocol deposit value to a bucket and recovered 105
value, it will increase its "net overperformance" value by 5.

If a solar farm underperforms, which means that it recovers less protocol
deposit value from the bucket than it distributed to the bucket, it will
decrease its "net overperformance" by the delta. If this causes the net
overperformance to go negative, it will add the difference to its accumulated
drawdown.

For example, if a solar farm currently has a net overperformance of 3, and it
contributed 100 protocol deposit value to a bucket, and it recovered 95
protocol deposit value from that bucket, it will:

+ subtract 3 from its net overperformance, resetting it to 0
+ add 2 to its accumulated drawdown

If a solar farm underperforms, it must also make changes to the performance
pool. That extra 2 that was added to the accumulated drawdown must also be
added to the "net protocol deposit value" of the performance pool, and an
amount equal to `2 * farm.assets_required / farm.protocol_deposit_value` must
be added to the "net assets" value of the performance pool.

In essence, because the underperformance of the solar farm caused the
accumulated drawdown to be increased by a value of 2, the farm had to send 2
protocol deposit value to the performance pool, as well as a proportional
amount of assets.

Each week, the amount of rewards that the solar farm collects is proportional
to the protocol deposit value that it recovers from the bucket. If the
accumulated drawdown of the solar farm is less than the original protocol
deposit value of the solar farm, then the solar farm will be able to collect
its own assets up until the net drawdown of the solar farm is equal to the
original protocol deposit value of the solar farm. When the solar farm is
collecting its own assets, it must increase its net drawdown by the amount of
protocol deposit value that it is collecting.

The amount of assets that the farm gets per unit of recovered protocol deposit
value is equal to `farm.assets_required / farm.protocol_deposit_value`.

If the net drawdown of the farm is equal to the original protocol deposit value
of the farm, then the farm must collect its rewards from the performance pool.
For each unit of protocol deposit value that it collects from the performance
pool, it must decrease its net overperformance by a value of 1, it must
decrease the net protocol deposit value of the performance pool by one, it must
decrease the net assets of the performance pool by a value of `pool.net_assets
/ pool.net_protocol_deposit_value`, and the farm will receive a number of
assets equal to `pool.net_assets / pool.net_protocol_deposit_value`.

This setup guarantees that after its final week, a solar farm will have an
accumulated drawdown that is exactly equal to its original protocol deposit
value, and it will have a net overperformance of zero.

### Algorithmic Data Structures

The algorithm itself operates on a handful of data structures:

```rs
pub struct RegionID {
    pub region_id: String,
    pub asset_id: String,
}

pub struct Region {
    pub first_week: u64,
    pub final_week: u64,
    pub buckets: HashMap<i64, Bucket>,
}

pub struct Bucket {
    pub total_deposits: BigInt,
    pub total_carbon_credits: BigInt,

    pub first_week_farms: Vec<String>,
    pub ongoing_farms: Vec<String>,
    pub last_week_farms: Vec<String>,

    pub farm_states: HashMap<String, FarmBucketState>,

    pub pool_net_assets: BigInt,
    pub pool_net_deposits: BigInt,
}

pub struct FarmBucketState {
    pub deposits_contributed: BigInt,
    pub carbon_credits_contributed: BigInt,

    pub accumulated_drawdown: BigInt,
    pub net_overperformance: BigInt,
    pub rewards_this_week: BigInt,
}
```

### Building the Initial Data Structures

The first step of the algorithm is to fill out the basic information for each
region and each bucket. This can be done with a single pass over the farms.
Before the pass starts, a `HashSet<String>` is created which tracks the IDs of
each farm that has been processed. An error is returned if the input contains
two farms with the same ID.

For each farm, we first check if the corresponding region exists. All of the
regions are tracked in a `HashMap<RegionID, Region>`, so we can create the
region ID that corresponds to the region defined by the farm, and then create a
new region if one doesn't exist.

If we are creating a new region, the first week of the region will be set equal
to the first week of this farm, and the last week of the region will be set
equal to `farm.first_week + farm.weeks_alive - 1`. Both the `first_week` and
`weeks_alive` values on all farms must be positive non-zero values that are
less than 2^12.

One bucket will be created per week for the region, and the `total_deposits`
and `total_carbon_credits` values for the bucket will be set. The
`total_deposits` for each bucket will be set equal to
`farm.protocol_deposit_value / farm.weeks_alive` and the `total_carbon_credits`
for each bucket will be set equal to `farm.weekly_carbon_credits`.

The farm then has to add itself to the appropriate vec in the bucket. If this
is the first bucket where the farm appears, it adds itself to
`first_week_farms`. If this is the last week where the farm appears, it adds
itself to `last_week_farms`, otherwise it adds itself to `ongoing_farms`.

The farm then creates a FarmBucketState for itself and adds it to the
`farm_states` field in the bucket. The `deposits_contributed` value is set to
`farm.protocol_deposit_value / farm.weeks_alive` and the
`carbon_credits_contributed` value is set to `farm.weekly_carbon_credits`. The
accumulated drawdown and net overperformance values are both set to zero -
those will be computed dynamically later.

If a farm is being added to an existing region, most of the steps are the same,
except that it will be expanding the `first_week` and `final_week` values as
necessary, creating the corresponding new buckets if necessary, and then
inserting itself into any buckets that already exist. When a farm inserts
itself into an existing bucket, it increments `total_deposits` and
`total_carbon_credits` by the appropriate amount, appends itself to the
appropriate list of farms for that bucket, and then creates a FarmBucketState
for itself.

After each farm has been processed, there should be a set of regions, each
region should have a set of buckets, and each bucket should have a bunch of
farms.

### Dynamically Computing Rewards

Up until this point, all of the `pool_net_deposits` and `pool_net_assets` and
`accumulated_drawdown` and `net_overperformance` and `rewards_this_week` values
have been initialized to 0. The algorithm will go region-by-region, and
bucket-by-bucket, and compute the intermediate values for each farm, which will
ultimately compute the `rewards_this_week` values for each farm.

The loop structure is roughly:

```
for each region,
    for each bucket,
        for each farm,
            run some code,
```

The regions can be processed in any order. The buckets must be processed in
numerical order. It's possible that a region has gaps, which means that the
`first_week` might be 100, the `final_week` might be 500, and there are no
bucket for weeks 250-300. In that case, the missing buckets are simply skipped.

The farms must also be processed in a deterministic order. The vectors for each
bucket are created in a deterministic order based on the order that farms are
provided in the input, so we can iterate over the farms for each bucket based
on the vectors in the bucket. First we iterate over the farms in the
`first_week_farms` vector, then we iterate over the farms in the
`ongoing_farms` vector, and finally we iterate over the farms in the
`last_week_farms` vector.

When the algorithm progresses to the next bucket in a region, it needs to grab
the `pool_net_deposits` and `pool_net_assets` values from the previous bucket.
If there is no previous bucket, it will leave the values set to 0.

When the algorithm processes the next farm in a bucket, it will need to grab
the `accumulated_drawdown` and `net_overperformance` values for the farm in the
previous bucket. If the farm did not appear in the previous bucket, then these
values will remain set to 0.

Then, each farm follows the steps provided in the progressive vault model
description. First the farm computes how much protocol deposit value it
recovers:

```
deposits_recovered = bucketState.carbon_credits_contributed * bucket.total_deposits / bucket.total_carbon_credits
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

If this is the final bucket for the farm, when all computations are done a
consistency check should be run to make sure that
`bucketState.accumulated_drawdown` is equal to `farm.protocol_deposit_value`,
and also that `bucketState.net_overperformance` is zero.

If this is the final farm for the final bucket in a region, or if the next
bucket in the region does not exist, then a consistency check should be
performed to verify that `bucket.pool_net_assets` and
`bucket.pool_net_deposits` are both zero.

### Creating the Output

After the algorithm has been run, there will be a bunch of regions, each with a
bunch of buckets, and each bucket will have a bunch of farms, and each farm
will have a `rewards_this_week` value.

The process for crafting the final output starts by iterating over each region
and establishing a global first week, as well as a global last week. Then for
each week that appears in the range [globalFirst, globalLast], the algorithm
will iterate over every region, look for the corresponding bucket for the week
in that region, skip the region if it's not there, and add the rewards for
every farm in the bucket if it is there. Any weeks where no region at all has a
bucket for that week will be omitted from the output.

## Special Case: CGP Leftovers

For the "cgp" region only, and for the "usdg" asset only, farms will get bonus
rewards for weeks where there are `cgp_leftovers`. For each protocol deposit
value that the farm recovers, it can add `cgp_leftovers[week_num] /
bucket.total_deposits` to its `rewards_this_week`. This addition does not have
any interaction with the other variables - it won't modify
`net_overperformance` or `accumulated_drawdown` or change any of the pool
state, it just directly increases the `rewards_this_week` value for each farm
proportional to the deposits that the farm recovered.
