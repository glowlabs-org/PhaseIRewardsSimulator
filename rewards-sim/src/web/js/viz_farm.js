(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util;
  const S = App.state.state;

  function updateFarmSelectionHighlight() {
    U.Es("#farmSummaryCards .card").forEach(c => {
      c.classList.toggle("selected", String(c.getAttribute("data-fid")) === String(S.selectedFarmId));
    });
  }

  function renderFarmHeadline(farmObj) {
    const h = U.E("#farmHeadline");
    h.classList.add("centered");
    h.innerHTML = "";
    const m = farmObj.meta || {};

    const entries = (farmObj.entries || []).slice().sort((a, b) => a.week - b.week);
    let totalRewards = 0n;
    let totalGlwInflationEarned = 0n;
    for (const e of entries) {
      totalRewards += U.toBI(e.st.rewardsThisWeek);
      totalGlwInflationEarned += App.vizWeek.computeFarmGlwEarned(e.b, e.st);
    }

    const card = document.createElement("div");
    card.className = "card highlight wide";
    card.innerHTML = `
      <div class="card-header">
        <div class="card-title">Farm ${U.renderFarmId(farmObj.fid)} Overview</div>
      </div>
      <div class="kv">
        <div>Total deposit<br><strong>${U.formatDollarsScaled(m.protocolDepositValue || 0)}</strong></div>
        <div>Assets required<br><strong>${U.formatTokensScaled(m.assetsRequired || 0, m.assetId || "glw")}</strong></div>
        <div>Total GLW inflation<br><strong>${U.formatGlwUnscaled(totalGlwInflationEarned)}</strong></div>
        <div>Total rewards<br><strong>${U.formatTokensScaled(totalRewards, m.assetId || "glw")}</strong></div>
      </div>
    `;
    h.appendChild(card);
  }

  function renderFarmDetails(farmObj) {
    renderFarmHeadline(farmObj);
    const d = U.E("#farmDetails");
    d.innerHTML = "";
    const entries = (farmObj.entries || []).sort((a,b) => a.week - b.week);

    for (const e of entries) {
      const b = e.b;
      const st = e.st;
      const depRecBI = App.vizWeek.computeDepositsRecovered(b.totalDeposits, st.impactAssetsContributed, b.totalImpactAssets);
      const parts = App.vizWeek.computePoolAndOwnTokens(e.comp, b, st, depRecBI);
      const glwThisWeek = App.vizWeek.computeFarmGlwEarned(b, st);

      const kind = e.week === farmObj.meta.firstWeek ? "first" : (e.week === farmObj.meta.finalWeek ? "last" : "ongoing");

      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Week ${e.week}</div>
          <span class="badge ${kind}">${kind}</span>
        </div>
        <div class="kv">
          <div>Total deposits<br><strong>${U.formatDollarsScaled(b.totalDeposits)}</strong></div>
          <div>Total impact assets<br><strong>${U.formatImpactScaled(b.totalImpactAssets)}</strong></div>

          <div>Farm deposits<br><strong>${U.formatDollarsScaled(st.depositsContributed)}</strong></div>
          <div>Farm impact assets<br><strong>${U.formatImpactScaled(st.impactAssetsContributed)}</strong></div>

          <div>Deposits recovered<br><strong>${U.formatDollarsScaled(depRecBI)}</strong></div>
          <div>Rewards this week<br><strong>${U.formatTokensScaled(st.rewardsThisWeek, farmObj.meta.assetId || "glw")}</strong></div>

          <div>From own vault<br><strong>${U.formatTokensScaled(parts.tokensFromOwn, farmObj.meta.assetId || "glw")}</strong></div>
          <div>From pool<br><strong>${U.formatTokensScaled(parts.tokensFromPool, farmObj.meta.assetId || "glw")}</strong></div>

          <div>GLW earned this week<br><strong>${U.formatGlwUnscaled(glwThisWeek)}</strong></div>
          <div>GLW inflation (total)<br><strong>${U.formatGlwUnscaled(b.glwInflation || 0)}</strong></div>

          <div>Accum. drawdown<br><strong>${U.formatDollarsScaled(st.accumulatedDrawdown)}</strong></div>
          <div>Net overperf.<br><strong>${U.formatDollarsScaled(st.netOverperformance)}</strong></div>

          <div>Pool net assets<br><strong>${U.formatTokensScaled(b.poolNetAssets, farmObj.meta.assetId || "glw")}</strong></div>
          <div>Pool net deposits<br><strong>${U.formatDollarsScaled(b.poolNetDeposits)}</strong></div>
        </div>
      `;
      d.appendChild(card);
    }
  }

  function renderPerFarm() {
    if (!S.diagnostics) return;
    const comp = App.vizWeek.getDiagnosticsComp();
    const farmMap = new Map();

    if (comp) {
      for (const b of comp.buckets || []) {
        for (const st of b.farmStates || []) {
          const fid = st.farmId;
          if (!farmMap.has(fid)) {
            const finfo = (comp.farms || []).find(x => x.farmId === fid) || {};
            farmMap.set(fid, { meta: { ...finfo }, entries: [] });
          }
          const rec = farmMap.get(fid);
          rec.entries.push({ comp, b, st, week: b.weekNumber });
        }
      }
    }

    // Sort farms by protocol deposit size (descending)
    const farmArr = Array.from(farmMap.entries()).map(([fid, v]) => ({ fid, ...v }))
      .sort((a,b)=>{
        const da = U.toBI(a.meta && a.meta.protocolDepositValue || 0);
        const db = U.toBI(b.meta && b.meta.protocolDepositValue || 0);
        if (da === db) {
          const an = Number(a.fid), bn = Number(b.fid);
          if (Number.isFinite(an) && Number.isFinite(bn)) return an - bn;
          return String(a.fid).localeCompare(String(b.fid));
        }
        return db > da ? 1 : -1;
      });

    const holder = U.E("#farmSummaryCards");
    holder.innerHTML = "";
    for (const f of farmArr) {
      const card = document.createElement("div");
      card.className = "card compact";
      card.style.cursor = "pointer";
      card.setAttribute("data-fid", String(f.fid));
      const deposit = f.meta && f.meta.protocolDepositValue ? U.formatDollarsScaled(f.meta.protocolDepositValue) : "$0.00";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Farm ${U.renderFarmId(f.fid)}</div>
          <div class="badge">${deposit}</div>
        </div>
      `;
      card.onclick = () => {
        S.selectedFarmId = String(f.fid);
        updateFarmSelectionHighlight();
        renderFarmDetails(f);
      };
      holder.appendChild(card);
    }
    if (farmArr.length) {
      S.selectedFarmId = String(farmArr[0].fid);
      updateFarmSelectionHighlight();
      renderFarmDetails(farmArr[0]);
    }
  }

  App.vizFarm = {
    renderPerFarm,
    renderFarmHeadline,
    renderFarmDetails
  };
})();