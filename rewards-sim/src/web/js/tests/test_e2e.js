(function () {
  "use strict";

  onReady(function () {
    harness.test("end-to-end: configure farms, simulate, verify compact lists and detailed views", async function () {
      await waitFor(() => qs("#farmCards .card:not(.add-card)").length >= 3, 5000);
      let farmsCount = qs("#farmCards .card:not(.add-card)").length;
      harness.assert.truthy(farmsCount >= 3, "expected 3 default farms");

      const farm3 = findCardByTitle("#farmCards", "Farm 3");
      harness.assert.truthy(!!farm3, "farm 3 should exist");
      const delBtn = findButtonByText(farm3, "Delete");
      click(delBtn);
      await waitFor(() => qs("#farmCards .card:not(.add-card)").length === 2, 3000);

      await configureFarmById(1, { firstWeek: 1, weeksAlive: 2, weeklyIA: 1, protocolDeposit: 100, assetPrice: 1 });
      await configureFarmById(2, { firstWeek: 1, weeksAlive: 2, weeklyIA: 1, protocolDeposit: 100, assetPrice: 1 });

      click(q("#simulateBtn"));
      await waitFor(() => {
        const s = text(q("#status"));
        return s === "Simulation complete." || s === "Simulation failed.";
      }, 15000);
      harness.assert.equal(text(q("#status")), "Simulation complete.", "simulation did not complete successfully");

      const simKey = "simulation::glw";
      if (typeof window.__SELECT_VIZ_COMP__ === "function") {
        window.__SELECT_VIZ_COMP__(simKey);
      } else {
        const sel = q("#vizCompSelect");
        if (sel) {
          sel.value = simKey;
          sel.dispatchEvent(new Event("change"));
        }
      }

      const warnLis = qs("#warnings li");
      harness.assert.truthy(warnLis.length === 0, "expected no warnings for this simple case");

      await waitFor(() => qs("#weekCards .card").length === 2, 3000);
      const wk1Card = findCardByTitle("#weekCards", "Week 1");
      const wk2Card = findCardByTitle("#weekCards", "Week 2");
      harness.assert.truthy(!!wk1Card && !!wk2Card, "expected Week 1 & Week 2 cards");

      click(wk1Card);
      await waitFor(() => q("#weekHeadline .card"), 2000);
      const headline = q("#weekHeadline .card");

      const tdRaw = getKvMetricRaw(headline, "Total deposits");
      const tiaRaw = getKvMetricRaw(headline, "Total impact assets");
      const gliRaw = getKvMetricRaw(headline, "GLW inflation");
      const pnaRaw = getKvMetricRaw(headline, "Pool net assets");

      if (tdRaw.indexOf("$") === -1) throw new Error("Total deposits headline should be in dollars");
      if (tiaRaw.indexOf(".") === -1 && tiaRaw.indexOf(",") === -1 && parseNum(tiaRaw) !== 2) {}
      if (gliRaw.toUpperCase().indexOf("GLW") === -1) throw new Error("GLW inflation headline should be in GLW");
      if (pnaRaw.toUpperCase().indexOf("GLW") === -1) throw new Error("Pool net assets headline should be in GLW");

      assertNumEqual(getKvMetric(headline, "Total deposits"), 100, "week1 total deposits");
      assertNumEqual(getKvMetric(headline, "Total impact assets"), 2, "week1 total impact assets");
      assertNumEqual(getKvMetric(headline, "GLW inflation"), 0, "week1 glw inflation");
      assertNumEqual(getKvMetric(headline, "Pool net assets"), 0, "week1 pool net assets");

      await waitFor(() => qs("#weekDetails .card").length >= 1, 3000);
      let wk1Details = qs("#weekDetails .card");
      harness.assert.equal(wk1Details.length, 2, "week1 expected 2 active farms");

      for (const c of wk1Details) {
        harness.assert.equal(getBadge(c), "first", "week1 farm badge");
        assertNumEqual(getKvMetric(c, "Deposits contributed"), 50, "wk1 deposits_contributed");
        assertNumEqual(getKvMetric(c, "Impact assets contributed"), 1, "wk1 ia_contributed");
        assertNumEqual(getKvMetric(c, "Accum. drawdown"), 50, "wk1 accumulated_drawdown");
        assertNumEqual(getKvMetric(c, "Net overperf."), 0, "wk1 net_overperformance");
        const rRaw = getKvMetricRaw(c, "Rewards this week");
        if (rRaw.toUpperCase().indexOf("GLW") === -1) throw new Error("Rewards should show GLW");
        assertNumEqual(getKvMetric(c, "Rewards this week"), 50, "wk1 rewards_this_week");
        const ownRaw = getKvMetricRaw(c, "From own vault");
        const poolRaw = getKvMetricRaw(c, "From pool");
        if (ownRaw.toUpperCase().indexOf("GLW") === -1) throw new Error("From own vault should show GLW");
        if (poolRaw.toUpperCase().indexOf("GLW") === -1) throw new Error("From pool should show GLW");
      }

      click(wk2Card);
      await waitFor(() => q("#weekHeadline .card"), 2000);
      const headline2 = q("#weekHeadline .card");
      assertNumEqual(getKvMetric(headline2, "Total deposits"), 100, "week2 total deposits");

      await waitFor(() => qs("#weekDetails .card").length >= 1, 3000);
      const wk2Details = qs("#weekDetails .card");
      harness.assert.equal(wk2Details.length, 2, "week2 should show 2 active farms");
      for (const c of wk2Details) {
        harness.assert.equal(getBadge(c), "last", "week2 farm badge");
        const rRaw = getKvMetricRaw(c, "Rewards this week");
        if (rRaw.toUpperCase().indexOf("GLW") === -1) throw new Error("Rewards should show GLW");
      }

      click(q("#tabFarm"));
      await waitFor(() => !q("#perFarm").classList.contains("hidden"), 2000);

      const farmSummaries = qs("#farmSummaryCards .card");
      harness.assert.equal(farmSummaries.length, 2, "expected 2 farm summary cards");

      const farm1Summary = farmSummaries.find(c => text(q(".card-title", c)) === "Farm 1");
      harness.assert.truthy(!!farm1Summary, "missing farm 1 summary card");
      const badge = q(".badge", farm1Summary);
      harness.assert.truthy(!!badge, "farm summary badge missing");
      const depRaw = text(badge);
      if (depRaw.indexOf("$") === -1) throw new Error("Farm deposit badge should be dollars");
      assertNumEqual(parseNum(depRaw), 100, "farm1 deposit summary");

      click(farm1Summary);
      await waitFor(() => q("#farmHeadline .card"), 2000);
      const fh = q("#farmHeadline .card");
      assertNumEqual(getKvMetric(fh, "Total deposit"), 100, "farm headline total deposit");
      const f1DetailCards = qs("#farmDetails .card").sort((a,b) => parseNum(text(q(".card-title", a))) - parseNum(text(q(".card-title", b))));
      harness.assert.equal(f1DetailCards.length, 2, "farm1 should have 2 week detail cards");

      const f1W1 = f1DetailCards[0];
      const f1W2 = f1DetailCards[1];
      harness.assert.equal(text(q(".card-title", f1W1)), "Week 1", "farm1 first detail should be week 1");
      harness.assert.equal(text(q(".card-title", f1W2)), "Week 2", "farm1 second detail should be week 2");
      harness.assert.equal(getBadge(f1W1), "first", "farm1 week1 badge");
      harness.assert.equal(getBadge(f1W2), "last", "farm1 week2 badge");

      function assertFarmWeekCard(card, label) {
        assertNumEqual(getKvMetric(card, "Total deposits"), 100, label + " total deposits");
        assertNumEqual(getKvMetric(card, "Farm deposits"), 50, label + " farm deposits");
        assertNumEqual(getKvMetric(card, "Total impact assets"), 2, label + " total impact assets");
        assertNumEqual(getKvMetric(card, "Farm impact assets"), 1, label + " farm impact assets");
        assertNumEqual(getKvMetric(card, "Deposits recovered"), 50, label + " deposits recovered");
        assertNumEqual(getKvMetric(card, "Pool net assets"), 0, label + " pool net assets");
        assertNumEqual(getKvMetric(card, "Pool net deposits"), 0, label + " pool net deposits");
        const rewards = getKvMetric(card, "Rewards this week");
        harness.assert.truthy(!Number.isNaN(rewards), label + " rewards not a number");
        assertNumEqual(rewards, 50, label + " rewards this week");
        const rewardsRaw = getKvMetricRaw(card, "Rewards this week");
        if (rewardsRaw.toUpperCase().indexOf("GLW") === -1) throw new Error(label + " rewards should show GLW");
      }
      assertFarmWeekCard(f1W1, "farm1 week1");
      assertFarmWeekCard(f1W2, "farm1 week2");

      try { window.__E2E_DONE__ = true; } catch (_) {}
    });
  });
})();