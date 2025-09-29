# User Specification

# V1 Preprocessor

A specification for distributing rewards to solar farms on Glow V2 Phase I

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
is considered to have placed rewards in buckets 50-257. If the V2 transition
happens on week 58, then the solar farm's `v1_lifespan` is 200 weeks.

To convert `v1_lifespan` to `weeks_alive` for the rewards simulator, you use
the function `FLOOR(100 * v1_lifespan / 208) + 1`.

After all of the solar farms have been ported from V1 to V2, some amount of
money will be left in the buckets from the early liquidity. There will also be
money left in the buckets from farms that got banned from Glow V1 for fraud.
The total amount of leftover money in each bucket is provided in the
`cgp_leftovers` map.

There is a special case where one farm potentially needs to be refunded. To
handle that special case, you can subtract a proportional amount from each week
of leftovers. It's an imprecise fudge, but it gets the job done.
