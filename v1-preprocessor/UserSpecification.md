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
    "96": "12345",
    "97": "23456"
  },
  "solarFarms": {
    "45-ab": {
      "firstRewardWeek": 34,
      "netWeeklyImpactAssets": 0.12,
      "rewardSplits": [
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
    "96": "12292",
    "97": "23403"
  },
  "solarFarms": [
    {
      "farmId": "45-ab",
      "assetId": "usdg",
      "regionId": "utah",
      "netWeeklyImpactAssets": "120000000000000000",
      "protocolDepositValue": "16000",
      "assetsRequired": "10000",
      "firstWeek": 96,
      "weeksAlive": 71,
      "rewardSplits": [
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

## Examples Note

To keep the examples concise, truncated examples are provided. For example, the
cgpLeftovers map only displays two values in the example, but in the actual
output there will be many more values.

## Invariants

Within each `rewardSplits` array, the sum of all the
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
set to "usdg" for all farms, the 'regionId' will be set to "cgp". The
'netWeeklyImpactAssets' values will match after a type conversion, the
'protocolDepositValue' and 'assetsRequired' values will both be initialized to
0, and the 'rewardSplits' will match.

The 'firstWeek' value will be initialized to 96, and the 'weeksAlive' value
will be initialized to `1+floor(float(208-96+firstRewardWeek)/2.08)`.

The type conversion for netWeeklyImpactAssets is a conversion from a floating
point value to a BigInt that has been scaled up by 1e18 times. For example, a
value of '0.12' in history file will become a value of '120000000000000000' in
the output file, rounding to the nearest value if necessary.

After that, the algorithm will iterate through all of the protocol deposits.
For each protocol deposit, it will check if the 'correspondingFarm' already
exists in the list of solar farms in the output. If it does not exist, the
protocol deposit is skipped (this is because the corresponding farm was evicted
without refund). If it does exist, the 'usdgProvided' value is added to both
the 'protocolDepositValue' and the 'assetsRequired' values of the output.
Finally, the protocol deposit is subtracted from the 'cgpLeftovers' map using
the following logic:

```
for i := protocolDeposit.weekProvided+16; i < protocolDeposit.weekProvided+208; i++ {
    if i < 96 {
        continue
    }
    cgpLeftovers[i] -= ceil(float(protocolDeposit.usdgProvided) / 192.0)
}
```

NOTE: if a protocol deposit has been skipped, it will not be subtracted from
the cgpLeftovers. If cgpLeftovers[i] does not exist, that's an error.

Each protocol deposit is associated with one solar farm, but there may be
multiple protocol deposits that point to the same solar farm. That is okay.
Each time a new protocol deposit points to a solar farm, the 'usdgProvided'
value of that protocol deposit is added to the 'protocolDepositValue' and
'assetsRequired' value of the corresponding farm in the output.

After iterating through all of the protocol deposits, the algorithm will
iterate through the 'migratingToUtah' array. For each farm in the array, the
algorithm will update the 'regionId' of the corresponding farm to "utah", and
it will update the 'protocolDepositValue' of the corresponding farm to be equal
to the 'updatedProtocolDepositValue', overwriting the previous value. The
'assetsRequired' value is left unchanged.

For the final output, any keys for cgpLeftovers that are strictly smaller than
96 will be removed.

One final cleanup must be performed. Due to dust, this algorithm will actually
cause cgpLeftovers to potentially be negative. As long as the value is larger
than or equal to -10, this is acceptable. However, a negative value cannot be
returned.  Instead, any negative value larger than -10 must be pruned from the
final output. If there is a negative value that is less than -10, that is an
error, because it is no longer considered to be dust.

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
