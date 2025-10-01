(function () {
  "use strict";

  function onReady(cb) {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", cb, { once: true });
    } else {
      cb();
    }
  }

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

  function uiDisplayNumberGeneric(x) {
    const n = Number(x) || 0;
    const abs = Math.abs(n);
    if (abs < 1000) {
      return (n >= 0 ? Math.floor(n * 100) : Math.ceil(n * 100)) / 100;
    } else {
      return n >= 0 ? Math.floor(n) : Math.ceil(n);
    }
  }

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

    const fw = getKvMetric(card, "First week");
    const wa = getKvMetric(card, "Weeks alive");
    if (!(fw === cfg.firstWeek && wa === cfg.weeksAlive)) {
      throw new Error("First week/Weeks alive values not displayed correctly in KV");
    }

    const kvItems = qs(".kv > div", card);
    const hasGLW = kvItems.some(it => text(it).toLowerCase().startsWith("glw price"));
    if (!hasGLW) throw new Error("GLW Price label missing");
    const priceText = getKvMetricRaw(card, "GLW Price");
    if (priceText.indexOf("$") === -1) throw new Error("GLW Price should include $");
  }

  try { window.__E2E_DONE__ = false; } catch (_) {}

  window.TEST_HELPERS = {
    onReady,
    waitFor, q, qs, click, text, parseNum,
    assertNumEqual, assertNumClose,
    findButtonByText, getKvMetric, getKvMetricRaw, getBadge, findCardByTitle,
    uiDisplayNumberGeneric, dollarsFromBI, tokensFromBI, configureFarmById
  };

  // Also expose as globals for convenience
  self.onReady = onReady;
  self.waitFor = waitFor;
  self.q = q;
  self.qs = qs;
  self.click = click;
  self.text = text;
  self.parseNum = parseNum;
  self.assertNumEqual = assertNumEqual;
  self.assertNumClose = assertNumClose;
  self.findButtonByText = findButtonByText;
  self.getKvMetric = getKvMetric;
  self.getKvMetricRaw = getKvMetricRaw;
  self.getBadge = getBadge;
  self.findCardByTitle = findCardByTitle;
  self.uiDisplayNumberGeneric = uiDisplayNumberGeneric;
  self.dollarsFromBI = dollarsFromBI;
  self.tokensFromBI = tokensFromBI;
  self.configureFarmById = configureFarmById;
})();