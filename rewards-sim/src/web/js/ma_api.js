(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util;
  const S = App.maState.state;

  // Scale Factors
  const SCALE_GLW = U.bigPow10(18);
  const SCALE_USD = U.bigPow10(6);
  const SCALE_USDG = U.bigPow10(6);
  const SCALE_SGCTL = U.bigPow10(6);

  function getAssetScale(assetId) {
    const id = String(assetId).toUpperCase();
    if (id === "GLW") return SCALE_GLW;
    if (id === "USDG") return SCALE_USDG;
    if (id === "SGCTL") return SCALE_SGCTL;
    return SCALE_GLW; // default
  }

  function getAssetDecimals(assetId) {
    const id = String(assetId).toUpperCase();
    if (id === "GLW") return 18;
    return 6;
  }

  function buildApiInput() {
    const solarFarms = S.farms.map(f => {
      const assets = f.assets.map(a => {
        // assetsRequiredUSDC: scale 1e6
        const usdc = BigInt(Math.round(a.amountUSD * 1000000));
        // price: scale 1e6
        const price = BigInt(Math.round(a.price * 1000000));
        
        // assetsRequired = (usdc * assetScale) / price
        // Note: usdc and price are both scaled by 1e6, so division cancels that out
        // result is in units, then multiply by assetScale.
        // Formula: (usdc * assetScale) / price
        let required = 0n;
        if (price > 0n) {
          required = (usdc * getAssetScale(a.assetId)) / price;
        }

        return {
          assetId: a.assetId,
          assetsRequired: required.toString(),
          assetsRequiredUSDC: usdc.toString(),
          quotedByGvePricePerAsset: price.toString(),
          decimals: getAssetDecimals(a.assetId)
        };
      });

      const totalDeposit = BigInt(Math.round(f.totalDeposit * 1000000));
      // Weekly Impact: scale 1e18
      // f.weeklyIA is float.
      const weeklyIA = U.toScaledIntString(f.weeklyIA, 18);

      return {
        farmId: String(f.id),
        regionId: parseInt(f.regionId),
        netWeeklyImpactAssets: weeklyIA,
        totalProtocolDepositValue: totalDeposit.toString(),
        assets: assets,
        firstWeek: parseInt(f.firstWeek),
        weeksAlive: parseInt(f.weeksAlive),
        rewardSplit: [
            {
              walletAddress: U.randomEthAddress(),
              glowSplitPercent6Decimals: "1000000",
              depositSplitPercent6Decimals: "1000000",
            },
        ]
      };
    });

    return {
      cgpLeftovers: {},
      solarFarms,
      gctlDistribution: {}
    };
  }

  async function simulate() {
    App.maDesigner.setStatus("Simulating...");
    S.simulationOutput = null;
    U.E("#warnings").innerHTML = "";
    U.E("#weekCards").innerHTML = "";
    U.E("#weekHeadline").innerHTML = "";
    U.E("#weekDetails").innerHTML = "";
    U.E("#farmSummaryCards").innerHTML = "";
    U.E("#farmHeadline").innerHTML = "";
    U.E("#farmDetails").innerHTML = "";
    S.selectedWeek = null;
    S.selectedFarmId = null;

    if (!S.farms.length) {
      App.maDesigner.setStatus("No farms to simulate.");
      return;
    }

    const body = buildApiInput();
    
    try {
      const preload = !!(U.E("#toggleV1") && U.E("#toggleV1").checked);
      const url = "/api/rewards-simulator-multi-asset" + (preload ? "?preloadGlowV1=true" : "");
      
      const res = await fetch(url, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      const data = await res.json();
      
      if (res.status === 200 || res.status === 422) {
        S.simulationOutput = data.output || data; // handle unwrapped or wrapped
        const errors = data.errors || [];
        
        if (errors.length) {
           U.E("#warnings").innerHTML = `<div class="card" style="border-left:4px solid var(--orange)"><strong>Errors/Warnings:</strong><ul>${errors.map(e => `<li>${U.escapeHtml(e)}</li>`).join("")}</ul></div>`;
        }

        App.maVizWeek.renderPerWeek();
        App.maVizFarm.renderPerFarm();
        App.maDesigner.setStatus("Simulation complete.");
      } else {
        App.maDesigner.setStatus("Simulation failed.");
        U.E("#warnings").innerHTML = `<div class="card" style="border-left:4px solid var(--orange)"><strong>Error:</strong> ${U.escapeHtml(data.error || "Unknown error")}</div>`;
      }

    } catch (err) {
      console.error(err);
      App.maDesigner.setStatus("Network error.");
      U.E("#warnings").innerHTML = `<div class="card" style="border-left:4px solid var(--orange)"><strong>Error:</strong> ${U.escapeHtml(String(err))}</div>`;
    }
  }

  App.maApi = {
    simulate
  };
})();