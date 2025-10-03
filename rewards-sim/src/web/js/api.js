(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util;
  const ST = App.state;
  const S = ST.state;

  const regionNameIdMap = new Map([
    ["cgp", 1],
    ["utah", 2],
    ["colorado", 3],
    ["missouri", 4],
  ]);

  function getRegionId(regionName) {
    const lower = String(regionName).toLowerCase();
    if (regionNameIdMap.has(lower)) {
      return regionNameIdMap.get(lower);
    }

    // Check if it's a user-defined region that we've already assigned an ID to.
    const comp = S.competitions.find((c) => c.regionId === regionName);
    if (comp && comp.regionNumericId) {
      return comp.regionNumericId;
    }

    // New user-defined region. Assign a new ID.
    const newId = S.nextCustomRegionId++;
    if (comp) {
      comp.regionNumericId = newId;
    }
    return newId;
  }

  function buildApiInput() {
    const solarFarms = [];
    for (const comp of S.competitions) {
      for (const f of comp.farms) {
        const wia = U.toScaledIntString(f.weeklyIA, 18);

        const pd = U.toScaledIntString(f.protocolDeposit, 6);
        const ap = U.toScaledIntString(Number(f.assetPrice).toFixed(2), 6);

        const pdBI = BigInt(pd);
        const apBI = BigInt(ap || "1");

        const tokenScale =
          String(comp.assetId).toLowerCase() === "usdg"
            ? U.bigPow10(6)
            : U.bigPow10(18);
        const arScaled = (pdBI * tokenScale) / (apBI === 0n ? 1n : apBI);

        solarFarms.push({
          farmId: String(f.id),
          assetId: comp.assetId,
          regionId: getRegionId(comp.regionId),
          netWeeklyImpactAssets: wia,
          protocolDepositValue: pd,
          assetsRequired: arScaled.toString(),
          rewardsAddress: U.randomEthAddress(),
          firstWeek: Number(f.firstWeek),
          weeksAlive: Math.max(2, Number(f.weeksAlive)),
        });
      }
    }

    return {
      cgpLeftovers: {},
      solarFarms,
    };
  }

  async function simulate() {
    App.designer.setStatus("Simulating...");
    S.diagnostics = null;
    U.E("#warnings").innerHTML = "";
    U.E("#weekCards").innerHTML = "";
    U.E("#weekHeadline").innerHTML = "";
    U.E("#weekDetails").innerHTML = "";
    U.E("#farmSummaryCards").innerHTML = "";
    U.E("#farmHeadline").innerHTML = "";
    U.E("#farmDetails").innerHTML = "";
    S.selectedWeek = null;
    S.selectedFarmId = null;

    const body = buildApiInput();
    if (!body.solarFarms.length) {
      App.designer.setStatus("Please add at least one farm.");
      return;
    }
    try {
      const preload = !!(U.E("#toggleV1") && U.E("#toggleV1").checked);
      const url =
        "/api/rewards-simulator-detailed" +
        (preload ? "?preloadGlowV1=true" : "");
      const res = await fetch(url, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      const data = await res.json();
      if (res.status === 200 || res.status === 422) {
        S.diagnostics = data;
        try {
          self.__DIAGNOSTICS__ = S.diagnostics;
        } catch (_) {}
        const errs = data.errors || [];
        if (errs.length) {
          const w = U.E("#warnings");
          w.innerHTML = `<div class="card" style="border-left:4px solid var(--orange)"><strong>Warnings:</strong><ul>${errs
            .map((e) => `<li>${U.escapeHtml(e)}</li>`)
            .join("")}</ul></div>`;
        }
        App.vizWeek.setupVizCompSelector();
        App.vizWeek.renderPerWeek();
        App.vizFarm.renderPerFarm();
        App.designer.setStatus("Simulation complete.");
      } else {
        App.designer.setStatus("Simulation failed.");
        const w = U.E("#warnings");
        w.innerHTML = `<div class="card" style="border-left:4px solid var(--orange)"><strong>Error:</strong> ${U.escapeHtml(
          data.error || "Unknown error"
        )}</div>`;
      }
    } catch (err) {
      App.designer.setStatus("Network error.");
      const w = U.E("#warnings");
      w.innerHTML = `<div class="card" style="border-left:4px solid var(--orange)"><strong>Error:</strong> ${U.escapeHtml(
        String(err)
      )}</div>`;
    }
  }

  App.api = {
    buildApiInput,
    simulate,
  };
})();