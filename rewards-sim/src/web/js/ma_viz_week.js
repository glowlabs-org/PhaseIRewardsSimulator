(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util;
  const S = App.maState.state;
  
  const WEEK_PAGE_SIZE = 12;
  const DETAILS_PAGE_SIZE = 6;

  function updateWeekSelectionHighlight() {
    U.Es("#weekCards .card").forEach(c => {
      c.classList.toggle("selected", String(c.getAttribute("data-week")) === String(S.selectedWeek));
    });
  }

  function renderWeekHeadline(weekNum, agg) {
    const h = U.E("#weekHeadline");
    h.classList.add("centered");
    h.innerHTML = "";
    
    const card = document.createElement("div");
    card.className = "card highlight wide";
    card.innerHTML = `
      <div class="card-header">
        <div class="card-title">Week ${weekNum} Overview</div>
      </div>
      <div class="kv">
        <div>Active Farms<br><strong>${agg.activeFarms}</strong></div>
        <div>Total GLW Inflation<br><strong>${U.formatTokensScaled(agg.totalGlwInflation, "glw")}</strong></div>
      </div>
    `;
    h.appendChild(card);
  }

  function renderWeekDetails(weekNum, agg) {
    renderWeekHeadline(weekNum, agg);
    const d = U.E("#weekDetails");
    d.innerHTML = "";
    
    const farms = agg.farms;
    const totalPages = Math.max(1, Math.ceil(farms.length / DETAILS_PAGE_SIZE));
    const start = S.weekFarmPage * DETAILS_PAGE_SIZE;
    const pageFarms = farms.slice(start, start + DETAILS_PAGE_SIZE);

    if (pageFarms.length === 0) {
      d.innerHTML = "<div class='card'>No active farms this week.</div>";
      return;
    }

    pageFarms.forEach(f => {
      const card = document.createElement("div");
      card.className = "card";
      
      let assetRewards = "";
      if (f.assets) {
        assetRewards = f.assets.map(a => 
          `<div class="glow-border">${a.assetId} Earned<br><strong>${U.formatTokensScaled(a.assetEarned, a.assetId)}</strong></div>`
        ).join("");
      }

      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">${U.escapeHtml(f.farmId)}</div>
          <span class="badge">Region ${f.regionId}</span>
        </div>
        <div class="kv">
          <div class="glow-border">GLW Inflation<br><strong>${U.formatTokensScaled(f.glowInflationReward, "glw")}</strong></div>
          ${assetRewards}
          <div>Protocol Deposit<br><strong>${U.formatDollarsScaled(f.protocolDeposit)}</strong></div>
          <div>Expected Prod.<br><strong>${U.formatImpactScaled(f.expectedProduction)}</strong></div>
        </div>
      `;
      d.appendChild(card);
    });

    if (totalPages > 1) {
      const pager = App.pager.create({
        totalItems: farms.length,
        pageSize: DETAILS_PAGE_SIZE,
        currentPage: S.weekFarmPage,
        windowSize: 5,
        onChange: (p) => {
          S.weekFarmPage = p;
          renderWeekDetails(weekNum, agg);
        }
      });
      d.appendChild(pager);
    }
  }

  function renderPerWeek() {
    if (!S.simulationOutput) return;
    
    // Output is map: week -> object
    // Object: { farmRewards: [], warnings: [] }
    // Note: if output came from API wrapped in "output", it's handled in api.js
    
    // Convert to sorted array of weeks
    const weeks = Object.keys(S.simulationOutput)
        .map(k => parseInt(k))
        .filter(k => !isNaN(k))
        .sort((a,b) => a - b);
    
    const weekCards = U.E("#weekCards");
    weekCards.innerHTML = "";
    
    const totalPages = Math.max(1, Math.ceil(weeks.length / WEEK_PAGE_SIZE));
    const start = S.weekPage * WEEK_PAGE_SIZE;
    const pageWeeks = weeks.slice(start, start + WEEK_PAGE_SIZE);
    
    pageWeeks.forEach(w => {
      const data = S.simulationOutput[String(w)];
      const rewards = data.farmRewards || [];
      const count = rewards.length;
      
      const card = document.createElement("div");
      card.className = "card compact";
      card.setAttribute("data-week", w);
      card.style.cursor = "pointer";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Week ${w}</div>
          <div class="badge">${count} farms</div>
        </div>
      `;
      card.onclick = () => {
        S.selectedWeek = w;
        S.weekFarmPage = 0;
        updateWeekSelectionHighlight();
        
        // Compute aggregation for details
        let totalInf = 0n;
        rewards.forEach(r => { totalInf += U.toBI(r.glowInflationReward); });
        
        renderWeekDetails(w, {
          activeFarms: count,
          totalGlwInflation: totalInf,
          farms: rewards
        });
      };
      weekCards.appendChild(card);
    });

    if (totalPages > 1) {
      const pager = App.pager.create({
        totalItems: weeks.length,
        pageSize: WEEK_PAGE_SIZE,
        currentPage: S.weekPage,
        windowSize: 7,
        onChange: (p) => {
          S.weekPage = p;
          renderPerWeek();
        }
      });
      weekCards.appendChild(pager);
    }

    if (pageWeeks.length > 0) {
      if (!S.selectedWeek || !pageWeeks.includes(S.selectedWeek)) {
        S.selectedWeek = pageWeeks[0];
      }
      // Trigger click logic for selected week
      const firstCard = weekCards.querySelector(`[data-week="${S.selectedWeek}"]`);
      if (firstCard) firstCard.click();
    }
  }

  App.maVizWeek = {
    renderPerWeek
  };
})();