(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util;

  const state = {
    competitions: [],
    selectedCompKey: null,

    diagnostics: null,
    selectedVizCompKey: null,

    addMode: false,
    nextId: 1,

    selectedWeek: null,
    selectedFarmId: null,
  };

  function initialFarms() {
    return [
      { id: String(state.nextId++), firstWeek: 1, weeksAlive: 5, weeklyIA: 0.08, protocolDeposit: 40000, assetPrice: 0.30, edit: false },
      { id: String(state.nextId++), firstWeek: 2, weeksAlive: 5, weeklyIA: 0.10, protocolDeposit: 80000, assetPrice: 0.40, edit: false },
      { id: String(state.nextId++), firstWeek: 2, weeksAlive: 5, weeklyIA: 0.12, protocolDeposit: 50000, assetPrice: 0.40, edit: false },
    ];
  }

  function ensureInitialData() {
    if (state.competitions.length) return;
    const comp = {
      regionId: "simulation",
      assetId: "glw",
      key: U.keyOf("simulation", "glw"),
      farms: initialFarms(),
    };
    state.competitions.push(comp);
    state.selectedCompKey = comp.key;
  }

  function defaultFarm() {
    return {
      id: String(state.nextId++),
      firstWeek: 1,
      weeksAlive: 5,
      weeklyIA: 0.08,
      protocolDeposit: 40000,
      assetPrice: 0.30,
      edit: false,
    };
  }

  function findComp(key) {
    return state.competitions.find(c => c.key === key) || null;
  }
  function currentComp() {
    return findComp(state.selectedCompKey);
  }
  function currentFarms() {
    const c = currentComp();
    return c ? c.farms : [];
  }

  function globalFarmIdExists(id, exceptObj) {
    for (const c of state.competitions) {
      for (const f of c.farms) {
        if (f === exceptObj) continue;
        if (String(f.id) === String(id)) return true;
      }
    }
    return false;
  }

  App.state = {
    state,
    ensureInitialData,
    defaultFarm,
    findComp,
    currentComp,
    currentFarms,
    globalFarmIdExists,
  };
})();