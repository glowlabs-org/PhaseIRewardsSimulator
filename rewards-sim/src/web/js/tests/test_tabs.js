(function () {
  "use strict";
  onReady(function () {
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
  });
})();