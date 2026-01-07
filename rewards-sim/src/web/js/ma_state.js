(function () {
  "use strict";
  const App = (self.App = self.App || {});

  const state = {
    farms: [],
    
    // Output data from simulation
    simulationOutput: null,
    
    selectedWeek: null,
    selectedFarmId: null,

    nextId: 3, // Start after defaults

    addMode: false,

    // Pagination
    weekPage: 0,
    weekFarmPage: 0,
    farmSummaryPage: 0,
    farmWeeksPage: 0,
  };

  function initialFarms() {
    return [
      {
        id: "farm-1",
        regionId: 3,
        firstWeek: 98,
        weeksAlive: 100,
        weeklyIA: 0.05,
        totalDeposit: 20000,
        assets: [
          { assetId: "GLW", price: 0.42, amountUSD: 20000 }
        ],
        edit: false
      },
      {
        id: "farm-2",
        regionId: 3,
        firstWeek: 98,
        weeksAlive: 100,
        weeklyIA: 0.0843,
        totalDeposit: 44641.79,
        assets: [
          { assetId: "GLW", price: 0.42, amountUSD: 26785.07 },
          { assetId: "USDG", price: 1.00, amountUSD: 11160.45 },
          { assetId: "SGCTL", price: 2.00, amountUSD: 6696.27 }
        ],
        edit: false
      }
    ];
  }

  function ensureInitialData() {
    if (state.farms.length === 0) {
      state.farms = initialFarms();
    }
  }

  function defaultFarm() {
    return {
      id: "farm-" + state.nextId++,
      regionId: 3,
      firstWeek: 98,
      weeksAlive: 100,
      weeklyIA: 0.05,
      totalDeposit: 20000,
      assets: [
        { assetId: "GLW", price: 0.42, amountUSD: 20000 }
      ],
      edit: false
    };
  }

  App.maState = {
    state,
    ensureInitialData,
    defaultFarm
  };
})();