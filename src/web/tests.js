(function () {
  "use strict";

  function waitFor(predicate, timeoutMs = 15000, intervalMs = 25) {
    return new Promise((resolve, reject) => {
      const deadline = Date.now() + timeoutMs;
      (function tick() {
        let ok = false;
        try { ok = !!predicate(); } catch (_e) {}
        if (ok) return resolve();
        if (Date.now() > deadline) return reject(new Error("waitFor timeout"));
        setTimeout(tick, intervalMs);
      })();
    });
  }

  function q(sel, root = document) { return root.querySelector(sel); }
  function qs(sel, root = document) { return Array.from(root.querySelectorAll(sel)); }

  function click(el) {
    if (!el) throw new Error("click: element not found");
    el.click();
  }

  function text(el) {
    return String(el && el.textContent || "").trim();
  }

  function parseNum(s) {
    const t = String(s).replace(/[^\d.\-]/g, "");
    if (!t.length) return NaN;
    const v = parseFloat(t);
    return v;
  }

  function assertNumEqual(actual, expected, msg) {
    const a = Number(actual);
    const e = Number(expected);
    if (!isFinite(a) || !isFinite(e)) {
      throw new Error((msg || "number equal") + " (not finite) " + a + " vs " + e);
    }
    if (Math.abs(a - e) > 1e-9) {
      throw new Error((msg || "number equal") + " expected " + e + " got " + a);
    }
  }

  function findButtonByText(root, txt) {
    txt = String(txt).toLowerCase();
    return qs("button", root).find(b => text(b).toLowerCase() === txt) || null;
  }

  function getKvMetric(cardEl, labelPrefix) {
    const items = qs(".kv > div", cardEl);
    for (const it of items) {
      if (text(it).toLowerCase().startsWith(String(labelPrefix).toLowerCase())) {
        const strong = q("strong", it);
        return parseNum(text(strong));
      }
    }
    return NaN;
  }

  function getBadge(cardEl) {
    const b = q(".badge", cardEl);
    return text(b);
  }

  function findCardByTitle(containerSel, titleText) {
    const cards = qs(containerSel + " .card");
    titleText = String(titleText);
    for (const c of cards) {
      const t = q(".card-title", c);
      if (t && text(t) === titleText) return c;
    }
    return null;
  }

  async function configureFarmById(fid, cfg) {
    // Find current card for the farm by title
    function findFarmCardExact(id, matchEdit) {
      const cards = qs("#farmCards .card:not(.add-card)");
      for (const c of cards) {
        const t = q(".card-title", c);
        if (!t) continue;
        const tt = text(t);
        const want = "Farm #" + String(id);
        if (matchEdit) {
          if (tt === want + " (edit)") return c;
        } else {
          if (tt === want) return c;
        }
      }
      return null;
    }

    // Open edit mode
    let card = findFarmCardExact(fid, false);
    if (!card) throw new Error("farm card not found for id=" + fid);

    const btnEdit = findButtonByText(card, "Edit");
    click(btnEdit);

    // After re-render, the original card node is stale. Reacquire edit card.
    await waitFor(() => !!findFarmCardExact(fid, true), 3000);
    card = findFarmCardExact(fid, true);
    const form = q(".inline-form", card);
    if (!form) throw new Error("edit form not found for id=" + fid);

    const setNum = (key, val) => {
      const inp = q('input[data-key="' + key + '"]', form);
      if (!inp) throw new Error("input not found: " + key);
      inp.value = String(val);
    };
    setNum("firstWeek", cfg.firstWeek);
    setNum("weeksAlive", cfg.weeksAlive);
    setNum("weeklyCC", cfg.weeklyCC);
    setNum("protocolDeposit", cfg.protocolDeposit);
    setNum("assetPrice", cfg.assetPrice);

    const btnSave = findButtonByText(card, "Save");
    click(btnSave);

    // Reacquire non-edit card after save
    await waitFor(() => !!findFarmCardExact(fid, false), 3000);
    card = findFarmCardExact(fid, false);

    // Verify summary subtitle reflects config
    const sub = q(".card-subtitle", card);
    const subText = text(sub);
    if (subText.indexOf("Week " + cfg.firstWeek) === -1 || subText.indexOf(cfg.weeksAlive + " weeks") === -1) {
      throw new Error("subtitle mismatch after save: " + subText);
    }
  }

  function sum(arr) { return arr.reduce((a, b) => a + b, 0); }

  window.addEventListener("load", function () {
    harness.test("visualizer boots", function () {
      harness.assert.truthy(q("#simulateBtn"), "missing #simulateBtn");
      harness.assert.truthy(q("#farmCards"), "missing #farmCards");
      harness.assert.truthy(q(".visualizer"), "missing .visualizer");
      harness.assert.truthy(q("#farmCards .card.add-card"), "missing add-card");
    });

    harness.test("end-to-end: configure farms, simulate, verify per-week and per-farm views", async function () {
      await waitFor(() => qs("#farmCards .card:not(.add-card)").length >= 1, 5000);
      let farmsCount = qs("#farmCards .card:not(.add-card)").length;
      harness.assert.truthy(farmsCount >= 1, "expected at least 1 farm");

      // Add second farm
      click(q("#farmCards .card.add-card"));
      await waitFor(() => qs("#farmCards .card:not(.add-card)").length >= 2, 5000);
      farmsCount = qs("#farmCards .card:not(.add-card)").length;
      harness.assert.equal(farmsCount, 2, "expected 2 farms after add");

      // Deterministic config for clean integers
      await configureFarmById(1, { firstWeek: 1, weeksAlive: 2, weeklyCC: 1, protocolDeposit: 100, assetPrice: 1 });
      await configureFarmById(2, { firstWeek: 1, weeksAlive: 2, weeklyCC: 1, protocolDeposit: 100, assetPrice: 1 });

      // Simulate
      click(q("#simulateBtn"));
      await waitFor(() => {
        const s = text(q("#status"));
        return s === "Simulation complete." || s === "Simulation failed.";
      }, 15000);
      harness.assert.equal(text(q("#status")), "Simulation complete.", "simulation did not complete successfully");

      // Expect no warnings for this simple balanced case
      const warnLis = qs("#warnings li");
      harness.assert.truthy(warnLis.length === 0, "expected no warnings for this simple case");

      // Per-week checks
      const weekCards = qs("#weekCards .card");
      harness.assert.equal(weekCards.length, 2, "expected 2 week cards");

      const wk1Card = findCardByTitle("#weekCards", "Week 1");
      const wk2Card = findCardByTitle("#weekCards", "Week 2");
      harness.assert.truthy(!!wk1Card && !!wk2Card, "expected Week 1 & Week 2 cards");

      assertNumEqual(getKvMetric(wk1Card, "Total deposits"), 100, "week1 total deposits");
      assertNumEqual(getKvMetric(wk1Card, "Total carbon"), 2, "week1 total carbon");
      assertNumEqual(getKvMetric(wk1Card, "Pool net assets"), 0, "week1 pool net assets");
      assertNumEqual(getKvMetric(wk1Card, "Pool net deposits"), 0, "week1 pool net deposits");

      // Week 1 details: both farms started -> 2 cards
      click(wk1Card);
      await waitFor(() => qs("#weekDetails .card").length >= 1, 3000);
      let wk1Details = qs("#weekDetails .card");
      harness.assert.equal(wk1Details.length, 2, "week1 expected 2 joining farms");

      for (const c of wk1Details) {
        harness.assert.equal(getBadge(c), "first", "week1 farm badge");
        assertNumEqual(getKvMetric(c, "Deposits contributed"), 50, "wk1 deposits_contributed");
        assertNumEqual(getKvMetric(c, "Carbon contributed"), 1, "wk1 carbon_contributed");
        assertNumEqual(getKvMetric(c, "Accum. drawdown"), 50, "wk1 accumulated_drawdown");
        assertNumEqual(getKvMetric(c, "Net overperf."), 0, "wk1 net_overperformance");
        assertNumEqual(getKvMetric(c, "Rewards this week"), 50, "wk1 rewards_this_week");
      }

      // Week 2 details: no new joiners
      click(wk2Card);
      await waitFor(() => qs("#weekDetails .card").length >= 1, 3000);
      const wk2DetailText = text(q("#weekDetails .card"));
      harness.assert.truthy(wk2DetailText.indexOf("No farms started this week.") !== -1, "week2 should show no joiners");

      // Per-farm checks
      click(q("#tabFarm"));
      await waitFor(() => !q("#perFarm").classList.contains("hidden"), 2000);

      const farmSummaries = qs("#farmSummaryCards .card");
      harness.assert.equal(farmSummaries.length, 2, "expected 2 farm summary cards");

      const farm1Summary = farmSummaries.find(c => text(q(".card-title", c)) === "Farm #1");
      harness.assert.truthy(!!farm1Summary, "missing farm #1 summary card");

      const totalRewardsF1 = getKvMetric(farm1Summary, "Total rewards");
      assertNumEqual(totalRewardsF1, 100, "farm1 total rewards summary");

      click(farm1Summary);
      await waitFor(() => qs("#farmDetails .card").length >= 1, 3000);
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
        assertNumEqual(getKvMetric(card, "Total carbon"), 2, label + " total carbon");
        assertNumEqual(getKvMetric(card, "Farm carbon"), 1, label + " farm carbon");
        assertNumEqual(getKvMetric(card, "Deposits recovered"), 50, label + " deposits recovered");
        assertNumEqual(getKvMetric(card, "Pool net assets"), 0, label + " pool net assets");
        assertNumEqual(getKvMetric(card, "Pool net deposits"), 0, label + " pool net deposits");
        const rewards = getKvMetric(card, "Rewards this week");
        harness.assert.truthy(!Number.isNaN(rewards), label + " rewards not a number");
        assertNumEqual(rewards, 50, label + " rewards this week");
      }
      assertFarmWeekCard(f1W1, "farm1 week1");
      assertFarmWeekCard(f1W2, "farm1 week2");

      const weeklyRewards = [
        getKvMetric(f1W1, "Rewards this week"),
        getKvMetric(f1W2, "Rewards this week"),
      ];
      assertNumEqual(sum(weeklyRewards), totalRewardsF1, "sum of weekly rewards equals summary total");
    });

    harness.test("tabs toggle views", async function () {
      const tabWeek = q("#tabWeek");
      const tabFarm = q("#tabFarm");
      const perWeek = q("#perWeek");
      const perFarm = q("#perFarm");

      harness.assert.truthy(tabWeek && tabFarm && perWeek && perFarm, "tabs or views missing");

      click(tabFarm);
      await waitFor(() => perFarm && !perFarm.classList.contains("hidden"), 1000);
      harness.assert.truthy(perWeek.classList.contains("hidden"), "perWeek should be hidden");

      click(tabWeek);
      await waitFor(() => perWeek && !perWeek.classList.contains("hidden"), 1000);
      harness.assert.truthy(perFarm.classList.contains("hidden"), "perFarm should be hidden");
    });
  });
})();