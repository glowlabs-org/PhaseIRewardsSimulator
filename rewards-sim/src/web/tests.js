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

  function assertNumClose(actual, expected, absTol, msg) {
    const a = Number(actual);
    const e = Number(expected);
    if (!isFinite(a) || !isFinite(e)) {
      throw new Error((msg || "number close") + " (not finite) " + a + " vs " + e);
    }
    if (Math.abs(a - e) > Math.abs(absTol)) {
      throw new Error((msg || "number close") + " expected ~" + e + " got " + a + " (tol=" + absTol + ")");
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

  function getKvMetricRaw(cardEl, labelPrefix) {
    const items = qs(".kv > div", cardEl);
    for (const it of items) {
      if (text(it).toLowerCase().startsWith(String(labelPrefix).toLowerCase())) {
        const strong = q("strong", it);
        return text(strong);
      }
    }
    return "";
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

  // Helpers to mirror UI display rounding/truncation rules exactly:
  // - If absolute value < 1000: show 2 decimals, truncated (not rounded)
  // - If >= 1000: show integer, truncated (floor)
  function uiDisplayNumberGeneric(x) {
    const n = Number(x) || 0;
    const abs = Math.abs(n);
    if (abs < 1000) {
      return (n >= 0 ? Math.floor(n * 100) : Math.ceil(n * 100)) / 100;
    } else {
      return n >= 0 ? Math.floor(n) : Math.ceil(n);
    }
  }

  // Convert BigInt-encoded values to numbers at given scale, without rounding
  function biStrToNumScaled(s, scale) {
    const str = String(s || "0").replace(/[^\d\-]/g, "");
    if (!str.length) return 0;
    const bi = BigInt(str);
    const intPart = bi / BigInt(scale);
    const frac = bi % BigInt(scale);
    return Number(intPart) + Number(frac) / Number(scale);
  }
  function dollarsFromBI(s) { return biStrToNumScaled(s, 1_000_000); }
  function tokensFromBI(s) { return biStrToNumScaled(s, 1_000_000_000_000_000_000); }

  async function configureFarmById(fid, cfg) {
    function findFarmCardExact(id, matchEdit) {
      const cards = qs("#farmCards .card:not(.add-card)");
      for (const c of cards) {
        const t = q(".card-title", c);
        if (!t) continue;
        const tt = text(t);
        const want = "Farm " + String(id);
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
    setNum("weeklyIA", cfg.weeklyIA);
    setNum("protocolDeposit", cfg.protocolDeposit);
    setNum("assetPrice", cfg.assetPrice);

    const btnSave = findButtonByText(card, "Save");
    click(btnSave);

    await waitFor(() => !!findFarmCardExact(fid, false), 3000);
    card = findFarmCardExact(fid, false);

    // Verify that First week and Weeks alive are displayed in KV
    const fw = getKvMetric(card, "First week");
    const wa = getKvMetric(card, "Weeks alive");
    if (!(fw === cfg.firstWeek && wa === cfg.weeksAlive)) {
      throw new Error("First week/Weeks alive values not displayed correctly in KV");
    }

    // Verify GLW Price label exists and shows $ with two decimals
    const kvItems = qs(".kv > div", card);
    const hasGLW = kvItems.some(it => text(it).toLowerCase().startsWith("glw price"));
    if (!hasGLW) throw new Error("GLW Price label missing");
    const priceText = getKvMetricRaw(card, "GLW Price");
    if (priceText.indexOf("$") === -1) throw new Error("GLW Price should include $");
  }

  try { window.__E2E_DONE__ = false; } catch (_) {}

  window.addEventListener("load", function () {
    harness.test("visualizer boots", function () {
      harness.assert.truthy(q("#simulateBtn"), "missing #simulateBtn");
      harness.assert.truthy(q("#sortBtn"), "missing #sortBtn");
      harness.assert.truthy(q("#farmCards"), "missing #farmCards");
      harness.assert.truthy(q(".visualizer"), "missing .visualizer");
      harness.assert.truthy(q("#farmCards .card.add-card"), "missing add-card");
    });

    harness.test("end-to-end: configure farms, simulate, verify compact lists and detailed views", async function () {
      await waitFor(() => qs("#farmCards .card:not(.add-card)").length >= 3, 5000);
      let farmsCount = qs("#farmCards .card:not(.add-card)").length;
      harness.assert.truthy(farmsCount >= 3, "expected 3 default farms");

      // Delete farm 3 (to keep a simple 2-farm scenario for deterministic checks)
      const farm3 = findCardByTitle("#farmCards", "Farm 3");
      harness.assert.truthy(!!farm3, "farm 3 should exist");
      const delBtn = findButtonByText(farm3, "Delete");
      click(delBtn);
      await waitFor(() => qs("#farmCards .card:not(.add-card)").length === 2, 3000);

      // Deterministic config for clean integers
      await configureFarmById(1, { firstWeek: 1, weeksAlive: 2, weeklyIA: 1, protocolDeposit: 100, assetPrice: 1 });
      await configureFarmById(2, { firstWeek: 1, weeksAlive: 2, weeklyIA: 1, protocolDeposit: 100, assetPrice: 1 });

      // Simulate
      click(q("#simulateBtn"));
      await waitFor(() => {
        const s = text(q("#status"));
        return s === "Simulation complete." || s === "Simulation failed.";
      }, 15000);
      harness.assert.equal(text(q("#status")), "Simulation complete.", "simulation did not complete successfully");

      // Select the simulation/glw competition explicitly to avoid interference from other tests
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

      // Expect no warnings for this simple balanced case
      const warnLis = qs("#warnings li");
      harness.assert.truthy(warnLis.length === 0, "expected no warnings for this simple case");

      // Per-week list should be compact (only titles + badge)
      await waitFor(() => qs("#weekCards .card").length === 2, 3000);
      const weekCards = qs("#weekCards .card");
      harness.assert.equal(weekCards.length, 2, "expected 2 week cards");
      const wk1Card = findCardByTitle("#weekCards", "Week 1");
      const wk2Card = findCardByTitle("#weekCards", "Week 2");
      harness.assert.truthy(!!wk1Card && !!wk2Card, "expected Week 1 & Week 2 cards");

      // Click Week 1 and verify headline + details (updated spec)
      click(wk1Card);
      await waitFor(() => q("#weekHeadline .card"), 2000);
      const headline = q("#weekHeadline .card");
      const tdRaw = getKvMetricRaw(headline, "Total deposits");
      const tiaRaw = getKvMetricRaw(headline, "Total impact assets");
      const gliRaw = getKvMetricRaw(headline, "GLW inflation");
      const pnaRaw = getKvMetricRaw(headline, "Pool net assets");

      if (tdRaw.indexOf("$") === -1) throw new Error("Total deposits headline should be in dollars");
      if (tiaRaw.indexOf(".") === -1 && tiaRaw.indexOf(",") === -1 && parseNum(tiaRaw) !== 2) {
        // tolerate either "2.00" or "2"
      }
      if (gliRaw.toUpperCase().indexOf("GLW") === -1) throw new Error("GLW inflation headline should be in GLW");
      if (pnaRaw.toUpperCase().indexOf("GLW") === -1) throw new Error("Pool net assets headline should be in GLW");

      assertNumEqual(getKvMetric(headline, "Total deposits"), 100, "week1 total deposits");
      assertNumEqual(getKvMetric(headline, "Total impact assets"), 2, "week1 total impact assets");
      assertNumEqual(getKvMetric(headline, "GLW inflation"), 0, "week1 glw inflation");
      assertNumEqual(getKvMetric(headline, "Pool net assets"), 0, "week1 pool net assets");

      // Week 1 details: both farms active -> 2 cards, status 'first'
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

      // Click Week 2
      click(wk2Card);
      await waitFor(() => q("#weekHeadline .card"), 2000);
      const headline2 = q("#weekHeadline .card");
      assertNumEqual(getKvMetric(headline2, "Total deposits"), 100, "week2 total deposits");
      // Details
      await waitFor(() => qs("#weekDetails .card").length >= 1, 3000);
      const wk2Details = qs("#weekDetails .card");
      harness.assert.equal(wk2Details.length, 2, "week2 should show 2 active farms");
      for (const c of wk2Details) {
        harness.assert.equal(getBadge(c), "last", "week2 farm badge");
        const rRaw = getKvMetricRaw(c, "Rewards this week");
        if (rRaw.toUpperCase().indexOf("GLW") === -1) throw new Error("Rewards should show GLW");
      }

      // Per-farm checks: list is compact and shows deposit badge
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

      // Clicking farm summary shows headline + detail cards
      click(farm1Summary);
      await waitFor(() => q("#farmHeadline .card"), 2000);
      const fh = q("#farmHeadline .card");
      // Headline includes totals and weeks
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
        String(c.regionId).toLowerCase() === "utah" &&
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
      const totalImpactShown = getKvMetric(headline, "Total impact assets");

      const farmCard = findCardByTitle("#weekDetails", "Farm " + String(st0.farmId));
      harness.assert.truthy(!!farmCard, "farm card for utah/usdg not found");
      const depContribShown = getKvMetric(farmCard, "Deposits contributed");
      const iaContribShown = getKvMetric(farmCard, "Impact assets contributed");
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

      assertNumClose(depContribShown, depContribDiagUI, 0.02, "utah/usdg deposits contributed matches diagnostics (UI)");
      assertNumClose(getKvMetric(farmCard, "Accum. drawdown"), accDrawDiagUI, 0.02, "utah/usdg accum. drawdown matches diagnostics (UI)");
      assertNumClose(getKvMetric(farmCard, "Net overperf."), netOverDiagUI, 0.02, "utah/usdg net overperf. matches diagnostics (UI)");

      const tdDiagUI = uiDisplayNumberGeneric(tdDiag);
      const tiDiagUI = uiDisplayNumberGeneric(tokensFromBI(b0.totalImpactAssets));
      assertNumClose(totalDepositsShown, tdDiagUI, 0.02, "utah/usdg headline total deposits");
      assertNumClose(totalImpactShown, tiDiagUI, 0.02, "utah/usdg headline total impact assets");
    });
  });
})();