(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util;
  const ST = App.state;

  function setupTabs() {
    const tabWeek = U.E("#tabWeek");
    const tabFarm = U.E("#tabFarm");
    const perWeek = U.E("#perWeek");
    const perFarm = U.E("#perFarm");

    if (!tabWeek || !tabFarm || !perWeek || !perFarm) return;

    tabWeek.onclick = () => {
      tabWeek.classList.add("active");
      tabFarm.classList.remove("active");
      perWeek.classList.remove("hidden");
      perFarm.classList.add("hidden");
    };
    tabFarm.onclick = () => {
      tabFarm.classList.add("active");
      tabWeek.classList.remove("active");
      perFarm.classList.remove("hidden");
      perWeek.classList.add("hidden");
    };
  }

  function init() {
    ST.ensureInitialData();
    setupTabs();
    App.designer.setupDesignerActions();
    App.designer.renderDesigner();

    const wc = U.E("#weekCards");
    if (wc) wc.classList.add("compact-grid");
    const fc = U.E("#farmSummaryCards");
    if (fc) fc.classList.add("compact-grid");
  }

  window.addEventListener("DOMContentLoaded", init);
})();