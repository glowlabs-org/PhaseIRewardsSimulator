(function () {
  "use strict";

  onReady(function () {
    harness.test("multi-asset e2e: configure farms, simulate, verify outputs", async function () {
      
      // 1. Check Initial State
      await waitFor(() => qs("#farmCards .card:not(.add-card)").length >= 2, 5000);
      const farms = qs("#farmCards .card:not(.add-card)");
      harness.assert.equal(farms.length, 2, "expected 2 initial farms");

      // Verify Farm 2 (multi-asset)
      const farm2 = findCardByTitle("#farmCards", "farm-2");
      harness.assert.truthy(farm2, "farm-2 not found");
      const kvText = text(farm2);
      harness.assert.truthy(kvText.includes("GLW"), "farm-2 should have GLW");
      harness.assert.truthy(kvText.includes("USDG"), "farm-2 should have USDG");
      harness.assert.truthy(kvText.includes("SGCTL"), "farm-2 should have SGCTL");

      // 2. Add a new Farm
      const addCard = q(".add-card");
      click(addCard);
      await waitFor(() => q(".add-card .inline-form"), 2000);
      const form = q(".add-card .inline-form");
      
      // Set values
      const setInput = (key, val) => {
        const inp = q(`input[data-key="${key}"]`, form);
        if(inp) { inp.value = val; inp.dispatchEvent(new Event("input")); }
      };
      
      setInput("id", "farm-test");
      setInput("totalDeposit", "1000"); // 1000 USD
      
      // The default asset is GLW 0 @ 0.
      // We need to configure it to match total deposit.
      const assetsContainer = q(".add-card div[style*='background']");
      const assetRow = q("div[style*='display: grid']", assetsContainer);
      const priceInp = assetRow.querySelectorAll("input")[0];
      const valInp = assetRow.querySelectorAll("input")[1];
      
      priceInp.value = "1"; priceInp.dispatchEvent(new Event("change"));
      valInp.value = "1000"; valInp.dispatchEvent(new Event("change"));
      
      // Check sum
      const sumCheck = q("#sum-check", assetsContainer);
      harness.assert.truthy(text(sumCheck).includes("OK"), "Sum check should be OK");

      const saveBtn = findButtonByText(q(".add-card"), "Save");
      click(saveBtn);

      await waitFor(() => qs("#farmCards .card:not(.add-card)").length === 3, 3000);

      // 3. Simulate
      click(q("#simulateBtn"));
      await waitFor(() => text(q("#status")) === "Simulation complete.", 10000);

      // 4. Verify Week View
      await waitFor(() => qs("#weekCards .card").length > 0, 3000);
      const weekCard = qs("#weekCards .card")[0];
      click(weekCard);
      
      await waitFor(() => qs("#weekDetails .card").length > 0, 3000);
      const details = qs("#weekDetails .card");
      // Should see farm-2 details
      const f2Detail = details.find(c => text(c).includes("farm-2"));
      harness.assert.truthy(f2Detail, "farm-2 details should be visible in week view");
      harness.assert.truthy(text(f2Detail).includes("GLW Earned"), "farm-2 should show GLW Earned");
      harness.assert.truthy(text(f2Detail).includes("USDG Earned"), "farm-2 should show USDG Earned");

      // 5. Verify Farm View
      click(q("#tabFarm"));
      await waitFor(() => !q("#perFarm").classList.contains("hidden"), 2000);
      await waitFor(() => qs("#farmSummaryCards .card").length >= 3, 3000);
      
      const f2Summary = qs("#farmSummaryCards .card").find(c => text(c).includes("farm-2"));
      click(f2Summary);
      
      await waitFor(() => q("#farmDetails .card"), 2000);
      const f2Weeks = qs("#farmDetails .card");
      harness.assert.truthy(f2Weeks.length > 0, "farm-2 should have week details");
      const w1 = f2Weeks[0];
      harness.assert.truthy(text(w1).includes("GLW Inflation"), "should show GLW Inflation");
      harness.assert.truthy(text(w1).includes("Deposit Contrib."), "should show Deposit Contrib");
    });
  });
})();