(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util;
  const S = App.state.state;

  function computeDepositsRecovered(totalDepositsBI, farmIABI, totalIABI) {
    const td = U.toBI(totalDepositsBI);
    const fia = U.toBI(farmIABI);
    const tia = U.toBI(totalIABI);
    if (tia === 0n) return 0n;
    return (td * fia) / tia;
  }

  function findPrevNetOver(comp, week, farmId) {
    const prevWeek = Number(week) - 1;
    const prevBucket = (comp.buckets || []).find(b => Number(b.weekNumber) === prevWeek);
    if (!prevBucket) return 0n;
    const st = (prevBucket.farmStates || []).find(s => String(s.farmId) === String(farmId));
    if (!st) return 0n;
    return U.toBI(st.netOverperformance);
  }

  function computePoolAndOwnTokens(comp, bucket, st, depRecBI) {
    const depositsContrib = U.toBI(st.depositsContributed);
    const curNetOver = U.toBI(st.netOverperformance);
    const prevNetOver = findPrevNetOver(comp, bucket.weekNumber, st.farmId);
    let baseOver = 0n;
    if (depRecBI > depositsContrib) {
      baseOver += depRecBI - depositsContrib;
    }
    baseOver += prevNetOver;
    baseOver -= curNetOver;
    if (baseOver < 0n) baseOver = 0n;

    const poolNetAssets = U.toBI(bucket.poolNetAssets);
    const poolNetDeposits = U.toBI(bucket.poolNetDeposits);
    let tokensFromPool = 0n;
    if (poolNetDeposits > 0n) {
      tokensFromPool = (baseOver * poolNetAssets) / poolNetDeposits;
    }
    const weekRewards = U.toBI(st.rewardsThisWeek);
    let tokensFromOwn = weekRewards - tokensFromPool;
    if (tokensFromOwn < 0n) tokensFromOwn = 0n;
    return { tokensFromPool, tokensFromOwn };
  }

  function computeFarmGlwEarned(bucket, st) {
    const glwInfl = U.toBI(bucket.glwInflation || 0);
    const depContrib = U.toBI(st.depositsContributed);
    const totalDeps = U.toBI(bucket.totalDeposits || 0);
    if (totalDeps === 0n) return 0n;
    return (glwInfl * depContrib) / totalDeps;
  }

  function setupVizCompSelector() {
    const sel = U.E("#vizCompSelect");
    if (!sel || !S.diagnostics) return;
    const comps = Array.isArray(S.diagnostics.competitions) ? S.diagnostics.competitions : [];
    sel.innerHTML = "";
    for (const c of comps) {
      const k = U.keyOf(c.regionId, c.assetId);
      const opt = document.createElement("option");
      opt.value = k;
      opt.textContent = `${c.regionId} / ${String(c.assetId).toUpperCase()}`;
      sel.appendChild(opt);
    }
    if (!S.selectedVizCompKey || !comps.find(c => U.keyOf(c.regionId, c.assetId) === S.selectedVizCompKey)) {
      S.selectedVizCompKey = comps.length ? U.keyOf(comps[0].regionId, comps[0].assetId) : null;
    }
    if (S.selectedVizCompKey) sel.value = S.selectedVizCompKey;
    sel.onchange = () => {
      S.selectedVizCompKey = sel.value;
      S.selectedWeek = null;
      S.selectedFarmId = null;
      renderPerWeek();
      App.vizFarm.renderPerFarm();
    };
    try {
      self.__SELECT_VIZ_COMP__ = function (key) {
        S.selectedVizCompKey = key;
        if (sel) sel.value = key;
        S.selectedWeek = null;
        S.selectedFarmId = null;
        renderPerWeek();
        App.vizFarm.renderPerFarm();
      };
    } catch (_) {}
  }

  function getDiagnosticsComp() {
    if (!S.diagnostics) return null;
    const comps = Array.isArray(S.diagnostics.competitions) ? S.diagnostics.competitions : [];
    if (!comps.length) return null;
    if (!S.selectedVizCompKey) return comps[0];
    return comps.find(c => U.keyOf(c.regionId, c.assetId) === S.selectedVizCompKey) || comps[0];
  }

  function updateWeekSelectionHighlight() {
    U.Es("#weekCards .card").forEach(c => {
      c.classList.toggle("selected", String(c.getAttribute("data-week")) === String(S.selectedWeek));
    });
  }

  function renderWeekHeadline(weekNumber, agg, comp) {
    const head = U.E("#weekHeadline");
    head.classList.add("centered");
    head.innerHTML = "";
    const wrap = document.createElement("div");
    wrap.className = "card highlight wide";
    const compAsset = comp && comp.assetId ? comp.assetId : "glw";
    wrap.innerHTML = `
      <div class="card-header">
        <div class="card-title">Week ${weekNumber} Overview</div>
      </div>
      <div class="kv">
        <div>Total deposits<br><strong>${U.formatDollarsScaled(agg.total_deposits)}</strong></div>
        <div>Total impact assets<br><strong>${U.formatImpactScaled(agg.total_impact)}</strong></div>
        <div>GLW inflation<br><strong>${U.formatGlwUnscaled(agg.glw_inflation)}</strong></div>
        <div>Pool net assets<br><strong>${U.formatTokensScaled(agg.pool_assets, compAsset)}</strong></div>
      </div>
    `;
    head.appendChild(wrap);
  }

  function renderWeekDetails(weekNumber, agg, comp) {
    renderWeekHeadline(weekNumber, agg, comp);
    const details = U.E("#weekDetails");
    details.innerHTML = "";

    const items = (agg && agg.items) ? agg.items : [];
    if (!items.length) {
      const none = document.createElement("div");
      none.className = "card";
      none.textContent = "No farms active this week.";
      details.appendChild(none);
      return;
    }

    // Sort farms by protocol deposit value (descending) for improved readability
    const sorted = items.slice().sort((a, b) => {
      const fa = (comp.farms || []).find(x => x.farmId === a.st.farmId) || {};
      const fb = (comp.farms || []).find(x => x.farmId === b.st.farmId) || {};
      const da = U.toBI(fa.protocolDepositValue || 0);
      const db = U.toBI(fb.protocolDepositValue || 0);
      if (da === db) {
        // tie-breaker: numeric id then lexicographic
        const an = Number(a.st.farmId), bn = Number(b.st.farmId);
        if (Number.isFinite(an) && Number.isFinite(bn)) return an - bn;
        return String(a.st.farmId).localeCompare(String(b.st.farmId));
      }
      return db > da ? 1 : -1;
    });

    for (const it of sorted) {
      const { bucket, st } = it;
      const finfo = (comp.farms || []).find(x => x.farmId === st.farmId);
      const assetId = finfo ? finfo.assetId : "glw";
      const kind = (weekNumber === (finfo && finfo.firstWeek) ? "first" : (weekNumber === (finfo && finfo.finalWeek) ? "last" : "ongoing"));

      const depRec = computeDepositsRecovered(agg.total_deposits, st.impactAssetsContributed, agg.total_impact);
      const parts = computePoolAndOwnTokens(comp, bucket, st, depRec);
      const glwEarned = computeFarmGlwEarned(bucket, st);
      const weeksRemaining = finfo ? Math.max(0, Number(finfo.finalWeek) - Number(weekNumber)) : 0;

      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Farm ${U.renderFarmId(st.farmId)}</div>
          <span class="badge ${kind}">${kind}</span>
        </div>
        <div class="kv">
          <div>Deposits contributed<br><strong>${U.formatDollarsScaled(st.depositsContributed)}</strong></div>
          <div>Impact assets contributed<br><strong>${U.formatImpactScaled(st.impactAssetsContributed)}</strong></div>

          <div>Deposits recovered<br><strong>${U.formatDollarsScaled(depRec)}</strong></div>
          <div>Rewards this week<br><strong>${U.formatTokensScaled(st.rewardsThisWeek, assetId)}</strong></div>

          <div>From own vault<br><strong>${U.formatTokensScaled(parts.tokensFromOwn, assetId)}</strong></div>
          <div>From pool<br><strong>${U.formatTokensScaled(parts.tokensFromPool, assetId)}</strong></div>

          <div>GLW earned this week<br><strong>${U.formatGlwUnscaled(glwEarned)}</strong></div>
          <div>Weeks remaining<br><strong>${weeksRemaining}</strong></div>

          <div>Accum. drawdown<br><strong>${U.formatDollarsScaled(st.accumulatedDrawdown)}</strong></div>
          <div>Net overperf.<br><strong>${U.formatDollarsScaled(st.netOverperformance)}</strong></div>
        </div>
      `;
      details.appendChild(card);
    }
  }

  function renderPerWeek() {
    if (!S.diagnostics) return;
    const comp = getDiagnosticsComp();
    const weeksMap = new Map();

    if (comp) {
      for (const b of comp.buckets || []) {
        const w = b.weekNumber;
        if (!weeksMap.has(w)) {
          weeksMap.set(w, {
            total_deposits: 0n,
            total_impact: 0n,
            pool_assets: 0n,
            pool_deposits: 0n,
            glw_inflation: 0n,
            participants: 0,
            items: []
          });
        }
        const agg = weeksMap.get(w);
        agg.total_deposits += U.toBI(b.totalDeposits);
        agg.total_impact += U.toBI(b.totalImpactAssets);
        agg.pool_assets += U.toBI(b.poolNetAssets);
        agg.pool_deposits += U.toBI(b.poolNetDeposits);
        agg.glw_inflation += U.toBI(b.glwInflation);
        const states = Array.isArray(b.farmStates) ? b.farmStates : [];
        agg.participants += states.length;
        for (const st of states) {
          agg.items.push({ comp, bucket: b, st });
        }
      }
    }

    const weekCards = U.E("#weekCards");
    weekCards.innerHTML = "";
    const sortedWeeks = Array.from(weeksMap.keys()).sort((a,b)=>a-b);

    for (const w of sortedWeeks) {
      const item = weeksMap.get(w);
      const card = document.createElement("div");
      card.className = "card compact";
      card.setAttribute("data-week", String(w));
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Week ${w}</div>
          <div class="badge">${item.participants} farms</div>
        </div>
      `;
      card.style.cursor = "pointer";
      card.onclick = () => {
        S.selectedWeek = String(w);
        updateWeekSelectionHighlight();
        renderWeekDetails(w, item, comp);
      };
      weekCards.appendChild(card);
    }
    if (sortedWeeks.length) {
      S.selectedWeek = String(sortedWeeks[0]);
      updateWeekSelectionHighlight();
      renderWeekDetails(sortedWeeks[0], weeksMap.get(sortedWeeks[0]), comp);
    }
  }

  App.vizWeek = {
    setupVizCompSelector,
    renderPerWeek,
    computeDepositsRecovered,
    computePoolAndOwnTokens,
    computeFarmGlwEarned,
    getDiagnosticsComp
  };
})();