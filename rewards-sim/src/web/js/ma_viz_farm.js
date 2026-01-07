(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util;
  const S = App.maState.state;

  const FARM_SUMMARY_PAGE_SIZE = 12;
  const FARM_WEEKS_PAGE_SIZE = 6;

  function updateFarmHighlight() {
    U.Es("#farmSummaryCards .card").forEach(c => {
      c.classList.toggle("selected", c.getAttribute("data-fid") === S.selectedFarmId);
    });
  }

  function renderFarmHeadline(fid, agg) {
    const h = U.E("#farmHeadline");
    h.classList.add("centered");
    h.innerHTML = "";

    const assetTotals = {};
    Object.keys(agg.totalAssets).forEach(k => {
      assetTotals[k] = U.formatTokensScaled(agg.totalAssets[k], k);
    });
    
    // Find static info from latest week entry or first
    const anyWeek = agg.weeks[0];
    const deposit = anyWeek ? anyWeek.protocolDeposit : "0";
    const prod = anyWeek ? anyWeek.expectedProduction : "0";
    const region = anyWeek ? anyWeek.regionId : "?";
    
    // Get asset breakdown from inputs if available
    const farmInput = S.farms.find(f => String(f.id) === String(fid));
    let assetBreakdownHtml = "";
    if (farmInput) {
       assetBreakdownHtml = farmInput.assets.map(a => 
         `<div>Req: ${a.assetId}<br><strong>${U.formatTokensScaled(U.toScaledIntString(a.amountUSD/a.price, 0 /*approx*/), "none")}</strong></div>`
       ).join("");
    }

    const card = document.createElement("div");
    card.className = "card highlight wide";
    card.innerHTML = `
      <div class="card-header">
        <div class="card-title">${U.escapeHtml(fid)} Overview</div>
        <div class="badge">Region ${region}</div>
      </div>
      <div class="kv">
        <div>Total Deposit<br><strong>${U.formatDollarsScaled(deposit)}</strong></div>
        <div>Weekly Impact<br><strong>${U.formatImpactScaled(prod)}</strong></div>
        <div>Total GLW Inf.<br><strong>${U.formatTokensScaled(agg.totalGlw, "glw")}</strong></div>
        ${Object.keys(assetTotals).map(k => `<div>Total ${k}<br><strong>${assetTotals[k]}</strong></div>`).join("")}
      </div>
    `;
    h.appendChild(card);
  }

  function renderFarmDetails(fid, agg) {
    renderFarmHeadline(fid, agg);
    
    const d = U.E("#farmDetails");
    d.innerHTML = "";
    
    const entries = agg.weeks.sort((a,b) => a.weekIndex - b.weekIndex);
    const totalPages = Math.max(1, Math.ceil(entries.length / FARM_WEEKS_PAGE_SIZE));
    const start = S.farmWeeksPage * FARM_WEEKS_PAGE_SIZE;
    const pageEntries = entries.slice(start, start + FARM_WEEKS_PAGE_SIZE);

    pageEntries.forEach(e => {
      const card = document.createElement("div");
      card.className = "card";
      
      let assetsHtml = "";
      if (e.assets) {
        assetsHtml = e.assets.map(a => 
          `<div class="glow-border">${a.assetId} Earned<br><strong>${U.formatTokensScaled(a.assetEarned, a.assetId)}</strong></div>`
        ).join("");
      }

      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Week ${e.weekIndex}</div>
        </div>
        <div class="kv">
          <div class="glow-border">GLW Inflation<br><strong>${U.formatTokensScaled(e.glowInflationReward, "glw")}</strong></div>
          ${assetsHtml}
          <div>Deposit Contrib.<br><strong>${U.formatDollarsScaled(e.protocolDeposit)}</strong></div>
          <div>Impact Contrib.<br><strong>${U.formatImpactScaled(e.expectedProduction)}</strong></div>
        </div>
      `;
      d.appendChild(card);
    });

    if (totalPages > 1) {
      const pager = App.pager.create({
        totalItems: entries.length,
        pageSize: FARM_WEEKS_PAGE_SIZE,
        currentPage: S.farmWeeksPage,
        windowSize: 5,
        onChange: (p) => {
          S.farmWeeksPage = p;
          renderFarmDetails(fid, agg);
        }
      });
      d.appendChild(pager);
    }
  }

  function renderPerFarm() {
    if (!S.simulationOutput) return;

    // Pivot data: FarmID -> { totalGlw, totalAssets: {}, weeks: [] }
    const farmMap = new Map();
    
    Object.keys(S.simulationOutput).forEach(weekKey => {
      const weekData = S.simulationOutput[weekKey];
      if (!weekData.farmRewards) return;
      
      weekData.farmRewards.forEach(fr => {
        if (!farmMap.has(fr.farmId)) {
          farmMap.set(fr.farmId, { totalGlw: 0n, totalAssets: {}, weeks: [] });
        }
        const rec = farmMap.get(fr.farmId);
        
        rec.totalGlw += U.toBI(fr.glowInflationReward);
        if (fr.assets) {
          fr.assets.forEach(a => {
            if (!rec.totalAssets[a.assetId]) rec.totalAssets[a.assetId] = 0n;
            rec.totalAssets[a.assetId] += U.toBI(a.assetEarned);
          });
        }
        rec.weeks.push(fr);
      });
    });

    const farmIds = Array.from(farmMap.keys()).sort();
    const holder = U.E("#farmSummaryCards");
    holder.innerHTML = "";
    
    const totalPages = Math.max(1, Math.ceil(farmIds.length / FARM_SUMMARY_PAGE_SIZE));
    const start = S.farmSummaryPage * FARM_SUMMARY_PAGE_SIZE;
    const pageFarms = farmIds.slice(start, start + FARM_SUMMARY_PAGE_SIZE);

    pageFarms.forEach(fid => {
      const agg = farmMap.get(fid);
      // Get deposit from first week entry
      const deposit = agg.weeks[0] ? agg.weeks[0].protocolDeposit : "0";
      
      const card = document.createElement("div");
      card.className = "card compact";
      card.setAttribute("data-fid", fid);
      card.style.cursor = "pointer";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">${U.escapeHtml(fid)}</div>
          <div class="badge">${U.formatDollarsScaled(deposit)}</div>
        </div>
      `;
      card.onclick = () => {
        S.selectedFarmId = fid;
        S.farmWeeksPage = 0;
        updateFarmHighlight();
        renderFarmDetails(fid, agg);
      };
      holder.appendChild(card);
    });

    if (totalPages > 1) {
      const pager = App.pager.create({
        totalItems: farmIds.length,
        pageSize: FARM_SUMMARY_PAGE_SIZE,
        currentPage: S.farmSummaryPage,
        windowSize: 7,
        onChange: (p) => {
          S.farmSummaryPage = p;
          renderPerFarm();
        }
      });
      holder.appendChild(pager);
    }
    
    if (pageFarms.length > 0) {
      if (!S.selectedFarmId || !farmMap.has(S.selectedFarmId)) {
        S.selectedFarmId = pageFarms[0];
      }
      const agg = farmMap.get(S.selectedFarmId);
      // Ensure we highlight and render detail
      // We need to defer slightly or re-select element since we just rendered
      setTimeout(() => {
        updateFarmHighlight();
        renderFarmDetails(S.selectedFarmId, agg);
      }, 0);
    }
  }

  App.maVizFarm = {
    renderPerFarm
  };
})();