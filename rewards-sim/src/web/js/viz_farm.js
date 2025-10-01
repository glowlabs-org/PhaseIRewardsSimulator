(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util;
  const S = App.state.state;

  const FARM_SUMMARY_PAGE_SIZE = 12;
  const FARM_WEEKS_PAGE_SIZE = 6;

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
    const entriesSorted = (farmObj.entries || []).slice().sort((a,b) => a.week - b.week);

    const totalPages = Math.max(1, Math.ceil(entriesSorted.length / FARM_WEEKS_PAGE_SIZE));
    const start = S.farmWeeksPage * FARM_WEEKS_PAGE_SIZE;
    const pageEntries = entriesSorted.slice(start, start + FARM_WEEKS_PAGE_SIZE);

    for (const e of pageEntries) {
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
          <div class="card-title" data-full-fid="${U.escapeHtml(String(farmObj.fid))}">Week ${e.week}</div>
          <span class="badge ${kind}">${kind}</span>
        </div>
        <div class="kv">
          <div class="glow-border">GLW earned this week<br><strong>${U.formatGlwUnscaled(glwThisWeek)}</strong></div>
          <div class="glow-border">Rewards this week<br><strong>${U.formatTokensScaled(st.rewardsThisWeek, farmObj.meta.assetId || "glw")}</strong></div>

          <div>Farm deposits<br><strong>${U.formatDollarsScaled(st.depositsContributed)}</strong></div>
          <div>Farm impact assets<br><strong>${U.formatImpactScaled(st.impactAssetsContributed)}</strong></div>

          <div>Deposits recovered<br><strong>${U.formatDollarsScaled(depRecBI)}</strong></div>
          <div>GLW inflation (total)<br><strong>${U.formatGlwUnscaled(b.glwInflation || 0)}</strong></div>

          <div>Total deposits<br><strong>${U.formatDollarsScaled(b.totalDeposits)}</strong></div>
          <div>Total impact assets<br><strong>${U.formatImpactScaled(b.totalImpactAssets)}</strong></div>

          <div>Pool net assets<br><strong>${U.formatTokensScaled(b.poolNetAssets, farmObj.meta.assetId || "glw")}</strong></div>
          <div>Pool net deposits<br><strong>${U.formatDollarsScaled(b.poolNetDeposits)}</strong></div>

          <div>From own vault<br><strong>${U.formatTokensScaled(parts.tokensFromOwn, farmObj.meta.assetId || "glw")}</strong></div>
          <div>From pool<br><strong>${U.formatTokensScaled(parts.tokensFromPool, farmObj.meta.assetId || "glw")}</strong></div>

          <div>Accum. drawdown<br><strong>${U.formatDollarsScaled(st.accumulatedDrawdown)}</strong></div>
          <div>Net overperf.<br><strong>${U.formatDollarsScaled(st.netOverperformance)}</strong></div>
        </div>
      `;
      d.appendChild(card);
    }

    if (totalPages > 1) {
      const pager = document.createElement("div");
      pager.className = "pager";
      const prev = document.createElement("button");
      prev.className = "btn btn-ghost";
      prev.textContent = "Prev";
      prev.disabled = S.farmWeeksPage <= 0;
      prev.onclick = () => { S.farmWeeksPage = Math.max(0, S.farmWeeksPage - 1); renderFarmDetails(farmObj); };
      const next = document.createElement("button");
      next.className = "btn btn-ghost";
      next.textContent = "Next";
      next.disabled = S.farmWeeksPage >= (totalPages - 1);
      next.onclick = () => { S.farmWeeksPage = Math.min(totalPages - 1, S.farmWeeksPage + 1); renderFarmDetails(farmObj); };
      const ind = document.createElement("span");
      ind.className = "page-indicator";
      ind.textContent = `Page ${S.farmWeeksPage + 1} of ${totalPages}`;
      pager.append(prev, ind, next);
      d.appendChild(pager);
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

    const totalPages = Math.max(1, Math.ceil(farmArr.length / FARM_SUMMARY_PAGE_SIZE));
    const start = S.farmSummaryPage * FARM_SUMMARY_PAGE_SIZE;
    const pageFarms = farmArr.slice(start, start + FARM_SUMMARY_PAGE_SIZE);

    for (const f of pageFarms) {
      const card = document.createElement("div");
      card.className = "card compact";
      card.style.cursor = "pointer";
      card.setAttribute("data-fid", String(f.fid));
      const deposit = f.meta && f.meta.protocolDepositValue ? U.formatDollarsScaled(f.meta.protocolDepositValue) : "$0.00";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title" data-full-fid="${U.escapeHtml(String(f.fid))}">Farm ${U.renderFarmId(f.fid)}</div>
          <div class="badge">${deposit}</div>
        </div>
      `;
      card.onclick = () => {
        S.selectedFarmId = String(f.fid);
        S.farmWeeksPage = 0;
        updateFarmSelectionHighlight();
        renderFarmDetails(f);
      };
      holder.appendChild(card);
    }

    if (totalPages > 1) {
      const pager = document.createElement("div");
      pager.className = "pager";
      const prev = document.createElement("button");
      prev.className = "btn btn-ghost";
      prev.textContent = "Prev";
      prev.disabled = S.farmSummaryPage <= 0;
      prev.onclick = () => { S.farmSummaryPage = Math.max(0, S.farmSummaryPage - 1); renderPerFarm(); };
      const next = document.createElement("button");
      next.className = "btn btn-ghost";
      next.textContent = "Next";
      next.disabled = S.farmSummaryPage >= (totalPages - 1);
      next.onclick = () => { S.farmSummaryPage = Math.min(totalPages - 1, S.farmSummaryPage + 1); renderPerFarm(); };
      const ind = document.createElement("span");
      ind.className = "page-indicator";
      ind.textContent = `Page ${S.farmSummaryPage + 1} of ${totalPages}`;
      pager.append(prev, ind, next);
      holder.appendChild(pager);
    }

    if (pageFarms.length) {
      if (!S.selectedFarmId || !pageFarms.find(x => String(x.fid) === String(S.selectedFarmId))) {
        S.selectedFarmId = String(pageFarms[0].fid);
      }
      updateFarmSelectionHighlight();
      const current = pageFarms.find(x => String(x.fid) === String(S.selectedFarmId)) || pageFarms[0];
      renderFarmDetails(current);
    }
  }

  App.vizFarm = {
    renderPerFarm,
    renderFarmHeadline,
    renderFarmDetails
  };
})();