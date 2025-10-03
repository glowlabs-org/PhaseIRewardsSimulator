(function () {
  "use strict";

  onReady(function () {
    harness.test("import v1: utah / USDG week/farm values consistent with diagnostics", async function () {
      await waitFor(() => window.__E2E_DONE__ === true, 30000);

      const v1 = q("#toggleV1");
      harness.assert.truthy(!!v1, "missing import v1 toggle");
      if (!v1.checked) click(v1);

      click(q("#simulateBtn"));
      await waitFor(() => {
        const s = text(q("#status"));
        return s === "Simulation complete." || s === "Simulation failed.";
      }, 20000);
      harness.assert.equal(text(q("#status")), "Simulation complete.", "simulation failed");

      await waitFor(() => !!window.__DIAGNOSTICS__ && Array.isArray(window.__DIAGNOSTICS__.competitions), 5000);
      const di = window.__DIAGNOSTICS__;
      const comps = di.competitions || [];
      const utahUsd = comps.find(c =>
        c.regionId === 2 && // Utah is region 2
        String(c.assetId).toLowerCase() === "usdg"
      );

      if (!utahUsd) {
        harness.log("utah/usdg competition not present in v1 data; skipping detailed checks");
        return;
      }

      const buckets = (utahUsd.buckets || []).slice().sort((a,b)=>Number(a.weekNumber)-Number(b.weekNumber));
      harness.assert.truthy(buckets.length > 0, "expected at least one bucket in utah/usdg");
      const b0 = buckets[0];
      harness.assert.truthy((b0.farmStates || []).length > 0, "expected at least one farm state in first utah/usdg bucket");
      const st0 = b0.farmStates[0];

      const sel = q("#vizCompSelect");
      harness.assert.truthy(!!sel, "viz select missing");
      const key = String(utahUsd.regionId) + "::" + String(utahUsd.assetId);
      if (typeof window.__SELECT_VIZ_COMP__ === "function") {
        window.__SELECT_VIZ_COMP__(key);
      } else {
        sel.value = key;
        sel.dispatchEvent(new Event("change"));
      }

      await waitFor(() => qs("#weekCards .card").length > 0, 5000);
      const wkCard = findCardByTitle("#weekCards", "Week " + String(b0.weekNumber));
      harness.assert.truthy(!!wkCard, "week card not found for utah/usdg week");
      click(wkCard);
      await waitFor(() => q("#weekHeadline .card") && qs("#weekDetails .card").length > 0, 3000);

      const headline = q("#weekHeadline .card");
      const totalDepositsShown = getKvMetric(headline, "Total deposits");
      const totalImpactShown = getKvMetric(headline, "Total impact");

      const farmCard = findCardByTitle("#weekDetails", "Farm " + String(st0.farmId));
      harness.assert.truthy(!!farmCard, "farm card for utah/usdg not found");
      const depContribShown = getKvMetric(farmCard, "Farm deposits");
      const iaContribShown = getKvMetric(farmCard, "Farm impact");
      const depRecoveredShown = getKvMetric(farmCard, "Deposits recovered");

      const tdDiag = dollarsFromBI(b0.totalDeposits);
      const tiDiag = tokensFromBI(b0.totalImpactAssets);
      const iaDiag = tokensFromBI(st0.impactAssetsContributed);
      const expectedRecovered = (tiDiag !== 0 ? (tdDiag * iaDiag) / tiDiag : 0);
      const expectedRecoveredUI = uiDisplayNumberGeneric(expectedRecovered);

      assertNumClose(depRecoveredShown, expectedRecoveredUI, 0.02, "utah/usdg deposits recovered relation");

      const depContribDiagUI = uiDisplayNumberGeneric(dollarsFromBI(st0.depositsContributed));
      const accDrawDiagUI = uiDisplayNumberGeneric(dollarsFromBI(st0.accumulatedDrawdown));
      const netOverDiagUI = uiDisplayNumberGeneric(dollarsFromBI(st0.netOverperformance));

      assertNumClose(depContribShown, depContribDiagUI, 0.02, "utah/usdg farm deposits matches diagnostics (UI)");
      assertNumClose(getKvMetric(farmCard, "Accum. drawdown"), accDrawDiagUI, 0.02, "utah/usdg accum. drawdown matches diagnostics (UI)");
      assertNumClose(getKvMetric(farmCard, "Net overperf."), netOverDiagUI, 0.02, "utah/usdg net overperf. matches diagnostics (UI)");

      const tdDiagUI = uiDisplayNumberGeneric(tdDiag);
      const tiDiagUI = uiDisplayNumberGeneric(tokensFromBI(b0.totalImpactAssets));
      assertNumClose(totalDepositsShown, tdDiagUI, 0.02, "utah/usdg headline total deposits");
      assertNumClose(totalImpactShown, tiDiagUI, 0.02, "utah/usdg headline total impact");
    });
  });
})();