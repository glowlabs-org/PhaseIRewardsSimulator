(function () {
  "use strict";
  onReady(function () {
    harness.test("visualizer boots", function () {
      harness.assert.truthy(q("#simulateBtn"), "missing #simulateBtn");
      harness.assert.truthy(q("#sortBtn"), "missing #sortBtn");
      harness.assert.truthy(q("#farmCards"), "missing #farmCards");
      harness.assert.truthy(q(".visualizer"), "missing .visualizer");
      harness.assert.truthy(q("#farmCards .card.add-card"), "missing add-card");
    });
  });
})();