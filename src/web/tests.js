(function () {
  "use strict";

  function waitFor(predicate, timeoutMs = 10000, intervalMs = 50) {
    return new Promise((resolve, reject) => {
      const deadline = Date.now() + timeoutMs;
      (function tick() {
        let ok = false;
        try { ok = !!predicate(); } catch (_e) {}
        if (ok) return resolve();
        if (Date.now() > deadline) return reject(new Error("waitFor timeout"));
        setTimeout(tick, intervalMs);
      })();
    });
  }

  window.addEventListener("load", function () {
    harness.test("math basic", function () {
      harness.assert.equal(2 + 2, 4);
    });

    harness.test("UI boots", function () {
      harness.assert.truthy(document.querySelector("#simulateBtn"), "missing #simulateBtn");
      harness.assert.truthy(document.querySelector("#farmCards"), "missing #farmCards");
      harness.assert.truthy(document.querySelector(".visualizer"), "missing .visualizer");
    });

    harness.test("simulate -> renders output", async function () {
      const btn = document.querySelector("#simulateBtn");
      harness.assert.truthy(btn, "simulate button not found");
      btn.click();

      await waitFor(() => {
        const s = document.querySelector("#status")?.textContent?.trim();
        return s === "Simulation complete." || s === "Simulation failed.";
      }, 15000);

      const status = document.querySelector("#status")?.textContent?.trim();
      harness.assert.equal(status, "Simulation complete.", "simulation did not complete successfully");

      const weekCards = document.querySelectorAll("#weekCards .card").length;
      const farmCards = document.querySelectorAll("#farmSummaryCards .card").length;
      harness.assert.truthy(weekCards > 0 || farmCards > 0, "no output cards rendered");
      harness.log(`# weekCards: ${weekCards}, # farmCards: ${farmCards}`);
    });
  });
})();
