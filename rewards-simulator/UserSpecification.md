# User Specification

rewards-simulator is a rust program that takes as input a json object
containing a list of solar farms and other data, and produces as output a list
of rewards that each solar farm would receive on Glow V2 Phase I for each week.

## Input and Output Structure

The input to this program is a json object containing a list of farms that
defines all of the farms that are enrolled in the rewards program. The input
defines what week the farm joins the rewards program, and how many weeks the
farm is participating in the rewards program. The farm needs to participate in
the rewards program for an integer number of weeks between 2 and 2^12
(inclusive), which allows for farms that are being ported from V1 to V2 to
define a shorter tenure.

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
  "cgp_leftovers": {
    "96": 235,
    "97": 367
  },
  "solar_farms": [
    {
      "farm_id": "45",
      "asset_id": "glw",
      "region_id": "cgp",
      "weekly_carbon_credits": [
        1,
        [
          1
        ]
      ],
      "protocol_deposit_value": [
        1,
        [
          10000
        ]
      ],
      "assets_required": [
        1,
        [
          25000
        ]
      ],
      "rewards_address": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
      "first_week": 96,
      "weeks_alive": 100
    },
    {
      "farm_id": "90",
      "asset_id": "usdg",
      "region_id": "utah",
      "weekly_carbon_credits": [
        1,
        [
          1
        ]
      ],
      "protocol_deposit_value": [
        1,
        [
          6000
        ]
      ],
      "assets_required": [
        1,
        [
          6000
        ]
      ],
      "rewards_address": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
      "first_week": 96,
      "weeks_alive": 60
    }
  ]
}
```

Note: the `farm_id` is a string so that fractionalized farms can be represented
with suffixes. For example `"45_frac_1"`. Duplicate farm IDs are invalid.

Note: the `rewards_address` must be a valid ethereum address.

Note: the `cgp_leftovers` is a map from week number to the amount of usdg that
was put into the corresponding bucket by the early liquidity contract. This map
is used to distribute bonus rewards to solar farms participating in the cgp
region with the usdg asset.

Note: for the sake of keeping things simple, the values provided in the example
above have not been scaled the same way that they would have been scaled in
production.

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
          "amount": [
            1,
            [
              250
            ]
          ],
          "rewards_address": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D"
        },
        {
          "farm_id": "90",
          "asset_id": "usdg",
          "region_id": "utah",
          "amount": [
            1,
            [
              100
            ]
          ],
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

An additional endpoint exists at localhost:35025/api/rewards-simulator-detailed
which returns the full internal state of the program. This means that the
return value has a list of competitions, and each competition has a list of
buckets, and each bucket has a list of farms, and the full suite of algorithmic
data structures are available in the output. This endpoint is usually used for
visualizations.

### Input Validation

Among other requirements mentioned elsewhere, the API checks that all numerical
values provided in the input are positive and non-zero.

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

### Algorithmic Data Structures

The algorithm itself operates on a handful of data structures:

```rs
pub struct CompetitionID {
    pub region_id: String,
    pub asset_id: String,
}

pub struct Competition {
    pub first_week: u64,
    pub final_week: u64,
    pub buckets: HashMap<u64, Bucket>,
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
competition and each bucket. This can be done with a single pass over the farms.
Before the pass starts, a `HashSet<String>` is created which tracks the IDs of
each farm that has been processed. An error is returned if the input contains
two farms with the same ID.

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
competition, and the `total_deposits` and `total_carbon_credits` values for
each bucket will be set. The `total_deposits` for each bucket will be set equal
to `farm.protocol_deposit_value / farm.weeks_alive` and the
`total_carbon_credits` for each bucket will be set equal to
`farm.weekly_carbon_credits`.

The farm then has to add itself to the appropriate vec in the bucket. If this
is the first bucket where the farm appears, it adds itself to
`first_week_farms`. If this is the last bucket where the farm appears, it adds
itself to `last_week_farms`, otherwise it adds itself to `ongoing_farms`.

The farm then creates a FarmBucketState for itself and adds it to the
`farm_states` field in the bucket. The `deposits_contributed` value is set to
`farm.protocol_deposit_value / farm.weeks_alive` and the
`carbon_credits_contributed` value is set to `farm.weekly_carbon_credits`. The
accumulated drawdown and net overperformance values are both set to zero -
those will be computed dynamically later.

If a farm is being added to an existing competition, most of the steps are the
same, except that it will be expanding the `first_week` and `final_week` values
as necessary, creating the corresponding new buckets if necessary, and then
inserting itself into any buckets that already exist. When a farm inserts
itself into an existing bucket, it increments `total_deposits` and
`total_carbon_credits` by the appropriate amount, appends itself to the
appropriate list of farms for that bucket, and then creates a FarmBucketState
for itself.

After each farm has been processed, there should be a set of competitions, each
competition should have a set of buckets, and each bucket should have a set of
farms.

### Dynamically Computing Rewards

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

### Creating the Output

After the algorithm has been run, there will be a bunch of competitions, each
with a bunch of buckets, and each bucket will have a bunch of farms, and each
farm will have a `rewards_this_week` value.

The process for crafting the final output starts by iterating over each
competition and establishing a global first week, as well as a global last
week. Then for each week that appears in the range [globalFirst, globalLast],
the algorithm will iterate over every competition, look for the corresponding
bucket for the week in that competition, skip the competition if it's not
there, and add the rewards for every farm in the bucket if it is there. Any
weeks where no competition at all has a bucket for that week will be omitted
from the output.

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

Use of floating points is not allowed.

### BigInt JSON Form

By default, the input and output both use num-bigint's array form to encode
BigInt values. The input BigInts can also be provided as strings or even as
numbers and they will be parsed correctly. The outputs can be configured to be
returned as strings if either the query parameter `bigintsAsStrings` or
`bigints_as_strings` is set to a truthy value.

## Special Case: CGP Leftovers

For only the competition in the "cgp" region with the "usdg" asset, farms will
get bonus rewards for weeks where there are `cgp_leftovers`. For each protocol
deposit value that the farm recovers, it can add `cgp_leftovers[week_num] /
bucket.total_deposits` to its `rewards_this_week`. This addition does not have
any interaction with the other variables - it won't modify
`net_overperformance` or `accumulated_drawdown` or change any of the pool
state, it just directly increases the `rewards_this_week` value for each farm
proportional to the deposits that the farm recovered.

## Competition Visualizer

The competition visualizer is a visualizer that is served by the server at
index.html. The source code for the visualizer is stored at /src/web/

The visualizer is implemented in pure javascript/html/css - there are no
dependencies, including no dependencies on node or typescript.

The visualizer is a frontend that allows the user to design a rewards
competition, then run the rewards simulation, then visually introspect all of
the output. The visualizer only supports visualizing one competition, and the
asset for that competition is GLW tokens.

The page is split horizontally. The top portion of the page contains the "input
designer", which allows the user to add farms to the competition, and the
bottom portion of the page contains the output visualization. The output
visualization has two views that the user can switch between, one view is a
per-week visualization, and one view is a per-farm visualization.

### The Input Designer

Each farm is its own visual card, and the user can configure the following
values for the farm:

+ The first week that the farm joins the competition
+ The number of weeks the farm is in the competition
+ The number of carbon credits the farm produces each week
+ The protocol deposit of the farm (denominated in dollars)
+ The GLW token price (denominated in dollars)

When the page loads, it shows 3 farms:

+ The joining weeks for the farms are week 1, week 2, and week 2 respectively
+ The weeks alive for the farms are 5 for each farm
+ the weekly CC are 0.08, 0.1, and 0.12 respectively
+ The deposits for the farms are $40,000, $80,000, and $50,000 respectively
+ The GLW price for the farms is $0.30, $0.40, and $0.40 respectively

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
endpoint, the `farm_id` is automatically assigned (each farm has an ID that
increments by 1), the `asset_id` is automatically set to "glw", the `region_id`
is automatically set to "simulation", the `weekly_carbon_credits` is scaled up
by a factor of 1e18 from what the user inputs, the protocol deposit value is
scaled up by a factor of 1e18 from the user inputs, the `assets_required` is
set equal to the protocol deposit divided by the asset price and then scaled up
by a factor of 1e18, the rewards address is randomized, and the `first_week`
and `weeks_alive` are set to the values provided by the user.

The user interface accepts floating point inputs from the user for all values
that are going to be scaled up when they are submitted to the API endpoint.

A 'simulate rewards' button exists at the bottom of the input designer which
will send all of the farms to the rewards-simulator-detailed endpoint and then
parse the response and present the rewards visualization.

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
+ the total carbon credits for that week
+ the number of farms participating in that week
+ the net assets in the pool
+ the net deposits in the pool

All numbers are presented to the user as scaled down floating point values,
with a sensible amount of precision.

Each farm card displays the following information:

+ The deposits contributed by the farm to that week
+ The carbon credits contributed by the farm to that week
+ The accumulated drawdown of the farm as of that week
+ The net overperformance of the farm as of that week
+ The deposits recovered by the farm in that week (denominated in dollars)
+ The rewards for the farm that week (denominated in GLW)
+ The number of GLW rewards recovered from the farm's own vault
+ The number of GLW rewards recovered from the pool

The deposits recovered will need to be calculated by the frontend using the
equation `total_deposits * carbon_credits_contributed / total_carbon_credits`

The number of GLW rewards recovered from the pool can be calculated with the
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

The number of GLW rewards recovered from the farm's own vault is equal to the
total rewards minus the glw recovered from the pool.

### The Per-Farm Visualization

The per-farm visualization shows all of the farms that were simulated. The top
of the visualization is a neatly compacted list of every farm, where each farm
only shows its total deposit. It must be emphasized that this view must be
compact, ideally more than a dozen farms can fit on each row, and dozens of
rows can fit on each page.

When a user clicks on a farm, a detailed view for that farm is shown below the
list of farms. The detailed view shows all of the key details for the farm, and
it also shows one card for each week. The following details are shown in each
weekly card:

+ the total deposits for that week
+ the total carbon credits for that week
+ the number of farms participating in that week
+ the net assets in the pool for that week
+ the net deposits in the pool for that week
+ The deposits contributed by the farm to that week
+ The carbon credits contributed by the farm to that week
+ The accumulated drawdown of the farm as of that week
+ The net overperformance of the farm as of that week
+ The deposits recovered by the farm in that week (denominated in dollars)
+ The rewards for the farm that week (denominated in GLW)
+ The number of GLW rewards recovered from the farm's own vault
+ The number of GLW rewards recovered from the pool

The deposits recovered will need to be calculated by the frontend using the
equation `total_deposits * carbon_credits_contributed / total_carbon_credits`

The number of GLW rewards recovered from the pool can be calculated with the
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

The number of GLW rewards recovered from the farm's own vault is equal to the
total rewards minus the glw recovered from the pool.

Clicking on another farm will update the view to show the details and week
cards for that farm.

### Displaying Numbers

For all numbers that are strictly less than 1,000, the UI should display the
number with 2 decimals of precision. For all numbers that are larger than or
equal to 1000, the numbers should be displayed with commas and there should be
0 decimals of precision.

### Testing

The competition visualizer is tested using headless chromium. The webapp itself
features a test harness, and the index.html page will load the test harness and
test code if the query parameter '?test=1' or '?runTests=true' is provided.

The file tests.js can be used to inspect the DOM and manipulate the webpage
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
