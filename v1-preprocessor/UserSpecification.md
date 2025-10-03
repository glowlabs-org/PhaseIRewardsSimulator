# User Specification

v1-preprocessor is a rust program that takes Glow V1 rewards history and
translates it to Glow V2 rewards configuration data. The V1 history is
presented as a JSON file, provided at the location assets/v1-history.json, and
the V2 configuration data is output to assets/v2-configuration.json

## V1 History Data Format

The v1 history data format has a few different fields that get supplied. The
first field is the "usdgPerWeek" field, which contains a mapping from week
number to the total number of USDG rewards that were available on Glow V1 for
that week.

The second field is the "solarFarms" field, which contains a map of solar farms
that were active during Glow V1. The key of each element in the map is the ID
for the farm, and the value contains the first week that the farm started
receiving rewards, the impact asset production of the farm, and the rewards
splits for the farm.

The next field is the "protocolDeposits" field, which contains a list of all of
the protocol deposits that were made throughout Glow V1. Each protocol deposit
has an amount of USDG, the week that the protocol deposit was made in, and the
ID of the solar farm that the protocol deposit is covering.

The final field is the "migratingToUtah" field, which contains a list of solar
farms that are being migrated from the cgp region to the utah region. Each
solar farm has data to indicate the ID of the farm being migrated, as well as
the value that needs to be used to overwrite the farm's existing
'protocolDepositValue'.

```json
{
  "usdgPerWeek": {
    "97": "12345",
    "98": "23456"
  },
  "solarFarms": {
    "45-ab": {
      "firstRewardWeek": 34,
      "netWeeklyImpactAssets": 0.12,
      "rewardSplit": [
        {
          "walletAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
          "glowSplitPercent6Decimals": "1000000",
          "depositSplitPercent6Decimals": "1000000"
        }
      ]
    }
  },
  "protocolDeposits": [
    {
      "correspondingFarm": "45-ab",
      "usdgProvided": "10000",
      "weekProvided": 32
    }
  ],
  "migratingToUtah": [
    {
      "farmId": "45-ab",
      "updatedProtocolDepositValue": "16000"
    }
  ]
}
```

## V2 Configuration Data Format

The configuration data has two components. The first is the `cgpLeftovers`,
which is a mapping from the week number to the number of USDG rewards for that
week that cannot be directly attributed to any specific solar farm.

The second component is a list of solar farms, where each solar farm has a
handful of fields related to how it performs in the competition and how it
receives rewards.

```json
{
  "cgpLeftovers": {
    "97": "23403"
  },
  "solarFarms": [
    {
      "farmId": "45-ab",
      "assetId": "USDG",
      "regionId": 2,
      "netWeeklyImpactAssets": "120000000000000000",
      "protocolDepositValue": "16000",
      "assetsRequired": "7384",
      "firstWeek": 97,
      "weeksAlive": 70,
      "rewardSplit": [
        {
          "walletAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
          "glowSplitPercent6Decimals": "1000000",
          "depositSplitPercent6Decimals": "1000000"
        }
      ]
    }
  ]
}
```

For the regionId, '1' is cgp, which is the default regionId for all farms. '2'
is utah, which is the final regionId for all farms that were migrated to utah.

Note: To keep the examples concise, truncated examples are provided. For
example, the cgpLeftovers map only displays two values in the example, but in
the actual output there will be many more values.

## Invariants

Within each `rewardSplit` array, the sum of all the
`glowSplitPercent6Decimals` values should be 1000000, and the sum of all
`depositSplitPercent6Decimals` values should also be 1000000. If that invariant
doesn't hold, an error needs to be thrown.

Within the "solarFarms" output list, each element must have a unique "farmId".

cgpLeftovers can never have a negative value in the final output. There may be
negative value dust before the final output is generated.

If a solar farm is listed in migratingToUtah but does not appear in the list of
solar farms, that is an error.

## Type Notes

The farmId is a string.

The numbers serialized as strings are BigInt numbers. The serialization for the
BigInts always uses strings for both input and output.

## Building the V2 Configuration Data

The algorithm for building the V2 configuration data starts by iterating over
the 'usdgPerWeek' field from the input, and creating a matching 'cgpLeftovers'
field in the output. The cgpLeftovers data will be progressively updated as
more of the inputs are processed.

Then, for each solar farm in the input, the algorithm creates a corresponding
solar farm in the output. The 'farmId' value will match, the 'assetId' will be
set to "USDG" for all farms, the 'regionId' will be set to 1. The
'netWeeklyImpactAssets' values will match after a type conversion, the
'protocolDepositValue' and 'assetsRequired' values will both be initialized to
0, and the 'rewardSplit' will match.

The 'firstWeek' value will be initialized to 97, and the 'weeksAlive' value
will be initialized to `1+floor(float(208-97+firstRewardWeek)/2.08)`.

The type conversion for netWeeklyImpactAssets is a conversion from a floating
point value to a BigInt that has been scaled up by 1e18 times. For example, a
value of '0.12' in history file will become a value of '120000000000000000' in
the output file, rounding to the nearest value if necessary.

After that, the algorithm will iterate through all of the protocol deposits.
For each protocol deposit, it will check if the 'correspondingFarm' already
exists in the list of solar farms in the output. If it does not exist, the
protocol deposit is skipped (this is because the corresponding farm was evicted
without refund). If it does exist, the 'protocolDepositValue', the
'assetsRequired' value, and the 'cgpLeftovers' value is updated according to
the following algorithm:

```
for i := protocolDeposit.weekProvided+16; i < protocolDeposit.weekProvided+208; i++ {
    if i < 98 {
        continue
    }
    cgpLeftovers[i] -= ceil(float(protocolDeposit.usdgProvided) / 192.0)
    correspondingFarm.protocolDepositValue += floor(float(protocolDeposit.usdgProvided) / 192.0)
    correspondingFarm.assetsRequired += floor(float(protocolDeposit.usdgProvided) / 192.0)
}
```

Note: if a protocol deposit has been skipped, it will not be subtracted from
the cgpLeftovers. If cgpLeftovers[i] does not exist, that's an error.

Note: Each protocol deposit is associated with one solar farm, but there may be
multiple protocol deposits that point to the same solar farm.

Note: It is intentional that weeks are skipped if `i < 98`. This is due to the
architectural structure of the v1 rewards system.

After iterating through all of the protocol deposits, the algorithm will
iterate through the 'migratingToUtah' array. For each farm in the array, the
algorithm will update the 'regionId' of the corresponding farm to 2, and it
will update the 'protocolDepositValue' of the corresponding farm to be equal to
the 'updatedProtocolDepositValue', overwriting the previous value. The
'assetsRequired' value is left unchanged.

Then, all keys that are equal to or less than 97 in the cgpLeftovers array will
be removed. After that, all keys in the cgpLeftovers array will be reduced by
1.

For example, if the cgpLeftovers map looked like this:

```
  "cgpLeftovers": {
    "96": "1000000000",
    "97": "2000000000",
    "98": "3000000000",
    "99": "4000000000",
    "100": "5000000000",
    "101": "6000000000",
    "102": "7000000000",
    "103": "8000000000"
  }
```

It will, after this step, look like this:
```
  "cgpLeftovers": {
    "97": "3000000000",
    "98": "4000000000",
    "99": "5000000000",
    "100": "6000000000",
    "101": "7000000000",
    "102": "8000000000"
  }
```

You will notice that the above example reflects two steps that have been taken.
The first step was to prune weeks 96 and 97, and the second step was to shift
down all of the remaining weeks, such that what used to be week 98 is now week
97.

After this, the cgpLeftovers values must be merged by a factor of 2.08. This
means that week 97 will be changed so that its value is equal to the sum
previous values of week 97, and 98, and 8% of 99. Week 98 will be changed so
that its value is equal to the sum previous values of 92% of week 99, 100% of
week 100, and 16% of week 101, and so on.

One final cleanup must be performed. Due to dust, this algorithm will actually
cause cgpLeftovers values to potentially be negative. As long as the value is
larger than or equal to -20, this is acceptable. However, a negative value
cannot be returned. Instead, any negative value larger than -20 must be pruned
from the final output. Zero values must also be pruned from the final output.
If there is a negative value that is less than -20, that is an error, because
it is no longer considered to be dust.

After those cleanup steps, our example would look like this:
```
  "cgpLeftovers": {
    "97": "7400000000", // 100% of 97 + 100% of 98 + 8% of 99
    "98": "11720000000", // 92% off 99 + 100% of 100 + 16% of 101
    "99": "13880000000" // 84% of 101 + 100% of 102 (no other values exist)
  }
```

## Precision

Any rounding errors due to explicit rounding or integer division are acceptable
and are considered to be dust.

## Error Handling

Because this is financial data, error handling should be hair-trigger. The code
should be written to be highly defensive, and anything unexpected and not
explicitly covered in the spec should immediately result in an error. All
errors should have detailed messaging explaining what went wrong.

## Output Sorting

The cgpLeftovers values are to be sorted in the output so that the keys appear
in numerical order, and the solar farms in the output are sorted so that they
appear in numerical order of their 'weeksAlive' value.
