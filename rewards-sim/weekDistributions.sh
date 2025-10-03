curl -X POST "http://127.0.0.1:35025/api/rewards-simulator-detailed?preloadGlowV1=true&week=97" \
  -H 'Content-Type: application/json' \
  -d '{
    "solarFarms": [
      {
        "farmId": "45-bb",
        "assetId": "USDG",
        "regionId": 1,
        "netWeeklyImpactAssets": "23000000000000000000",
        "protocolDepositValue": "10000000",
        "assetsRequired": "10000000",
        "firstWeek": 98,
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
  }' > out.txt
