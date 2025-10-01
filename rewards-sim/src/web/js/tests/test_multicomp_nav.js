(function () {
  "use strict";

  onReady(function () {
    harness.test("multicomp nav: per-farm pagination state resets", async function () {
      // This test relies on the e2e test having run and enabled v1 import.
      await waitFor(() => window.__E2E_DONE__ === true, 30000);
      
      const v1 = q("#toggleV1");
      if (!v1 || !v1.checked) {
        harness.log("V1 import not enabled, skipping multicomp nav test");
        return;
      }

      await waitFor(() => !!window.__DIAGNOSTICS__ && Array.isArray(window.__DIAGNOSTICS__.competitions), 5000);
      
      const sel = q("#vizCompSelect");
      if (!sel || sel.options.length < 2) {
        harness.log("Not enough competitions to test nav, skipping");
        return;
      }
      
      // Find a competition with more than one page of farms.
      const di = window.__DIAGNOSTICS__;
      const FARM_SUMMARY_PAGE_SIZE = 12; // from viz_farm.js
      let bigCompKey = null, smallCompKey = null;

      for (const c of di.competitions) {
        const key = App.util.keyOf(c.regionId, c.assetId);
        const farmIds = new Set();
        for (const b of c.buckets || []) {
          for (const s of b.farmStates || []) {
            farmIds.add(s.farmId);
          }
        }
        const farmCount = farmIds.size;

        if (farmCount > FARM_SUMMARY_PAGE_SIZE) {
          bigCompKey = key;
        } else if (farmCount > 0 && farmCount <= FARM_SUMMARY_PAGE_SIZE) {
          smallCompKey = key;
        }
      }

      if (!bigCompKey || !smallCompKey) {
        harness.log("Could not find suitable large/small competitions, skipping");
        return;
      }

      // Switch to per-farm view
      click(q("#tabFarm"));
      await waitFor(() => !q("#perFarm").classList.contains("hidden"), 2000);

      // 1. Go to big competition
      window.__SELECT_VIZ_COMP__(bigCompKey);
      await waitFor(() => qs("#farmSummaryCards .card").length > 0, 3000);
      
      // 2. Go to page 2
      const pager = q("#farmSummaryCards .pagination");
      harness.assert.truthy(pager, "pager not found for big competition");
      const page2 = findButtonByText(pager, "2");
      harness.assert.truthy(page2, "page 2 button not found for big competition");
      click(page2);
      
      await waitFor(() => q("#farmSummaryCards .page-chip.active").textContent.trim() === "2", 1000);
      
      // 3. Switch to small competition
      window.__SELECT_VIZ_COMP__(smallCompKey);

      // 4. Assert farms are visible
      await waitFor(() => qs("#farmSummaryCards .card").length > 0, 3000, 50);
      const farmCards = qs("#farmSummaryCards .card");
      harness.assert.truthy(farmCards.length > 0, "farms should be visible on small competition after navigating on big one");
    });
  });
})();