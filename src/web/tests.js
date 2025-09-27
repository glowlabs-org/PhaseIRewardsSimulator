/**
 * Frontend smoke tests for the Glow Rewards Simulator UI.
 * - No external deps.
 * - Deterministic and quick.
 * - Uses the harness sink only.
 */
(function () {
  "use strict";

  /**
   * Wait for a condition to become true.
   * @param {Function} predicate - returns truthy when ready
   * @param {number} timeoutMs - max wait in ms
   * @param {number} intervalMs - poll interval in ms
   * @returns {Promise<void>}
   */
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
    // 1) Basic sanity
    harness.test("math basic", function () {
      harness.assert.equal(2 + 2, 4);
    });

    // 2) UI boots: check actual elements present in index.html
    harness.test("UI boots", function () {
      harness.assert.truthy(document.querySelector("#simulateBtn"), "missing #simulateBtn");
      harness.assert.truthy(document.querySelector("#farmCards"), "missing #farmCards");
      harness.assert.truthy(document.querySelector(".visualizer"), "missing .visualizer");
    });

    // 3) Simulation runs end-to-end against local API and renders output
    harness.test("simulate -> renders output", async function () {
      const btn = document.querySelector("#simulateBtn");
      harness.assert.truthy(btn, "simulate button not found");
      btn.click();

      // Wait until the status flips to either "Simulation complete." or "Simulation failed."
      await waitFor(() => {
        const s = document.querySelector("#status")?.textContent?.trim();
        return s === "Simulation complete." || s === "Simulation failed.";
      }, 15000);

      const status = document.querySelector("#status")?.textContent?.trim();
      harness.assert.equal(status, "Simulation complete.", "simulation did not complete successfully");

      // Expect some rendered content in either per-week or per-farm views
      const weekCards = document.querySelectorAll("#weekCards .card").length;
      const farmCards = document.querySelectorAll("#farmSummaryCards .card").length;
      harness.assert.truthy(weekCards > 0 || farmCards > 0, "no output cards rendered");
      harness.log(`# weekCards: ${weekCards}, # farmCards: ${farmCards}`);
    });
  });
})();
