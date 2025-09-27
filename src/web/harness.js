/**
 * Minimal DOM-based test harness.
 *
 * Design goals:
 *  - No external dependencies (no node, no npm).
 *  - Writes *only* harness output to a hidden <pre id="__TEST_OUTPUT__">.
 *  - Encodes final status in <html data-test-status="passed|failed"> for easy parsing.
 *  - Works with async tests; auto-finishes when all registered tests complete.
 *  - Can also be finished manually via harness.finish() for custom flows.
 *
 * Usage pattern in tests:
 *   harness.test("adds", () => {
 *     harness.assert.equal(1+1, 2, "math works");
 *   });
 *
 *   harness.test("async example", async () => {
 *     const r = await fetch("/ping").then(r => r.text());
 *     harness.assert.equal(r, "pong");
 *   });
 *
 *   // Optional if you want explicit control; otherwise auto-finishes when all tests complete
 *   // harness.finish();
 */
(function () {
  "use strict";

  /** @returns {HTMLElement} hidden <pre> where harness-only logs go */
  function ensureOutputEl() {
    /**
     * Creates or returns the hidden output element that captures the harness logs.
     * The element is hidden so it won’t interfere with the app UI.
     */
    var existing = document.getElementById("__TEST_OUTPUT__");
    if (existing) return existing;

    var el = document.createElement("pre");
    el.id = "__TEST_OUTPUT__";
    el.style.whiteSpace = "pre-wrap";
    el.style.display = "none";

    var attach = function () {
      (document.body || document.documentElement).appendChild(el);
    };
    if (document.body) attach();
    else document.addEventListener("DOMContentLoaded", attach, { once: true });

    return el;
  }

  /** @param {string} line */
  function write(line) {
    /**
     * Appends a single line to the harness-only output buffer.
     * This is what build.sh will print to stdout.
     */
    output.textContent += line + "\n";
  }

  /** @param {"passed" | "failed"} status */
  function setStatus(status) {
    /**
     * Sets the overall test status on <html> for easy parsing by build.sh,
     * and also prefixes the document title for humans.
     */
    document.documentElement.setAttribute("data-test-status", status);
    try {
      document.title = "[TEST-" + status.toUpperCase() + "] " + (document.title || "");
    } catch (_) { /* ignore if CSP/title issues */ }
  }

  /** Simple assertions with clear error messages. */
  var assert = {
    /**
     * Assert strict equality.
     * @param {*} a
     * @param {*} b
     * @param {string=} msg
     */
    equal: function (a, b, msg) {
      if (a !== b) throw new Error(msg || ("Expected ===\n  left: " + String(a) + "\n right: " + String(b)));
    },
    /**
     * Assert truthiness.
     * @param {*} x
     * @param {string=} msg
     */
    truthy: function (x, msg) {
      if (!x) throw new Error(msg || ("Expected truthy, got: " + String(x)));
    },
    /**
     * Assert that function throws.
     * @param {Function} fn
     * @param {RegExp|string=} match optional match on error message
     * @param {string=} msg
     */
    throws: function (fn, match, msg) {
      var threw = false, err;
      try { fn(); } catch (e) { threw = true; err = e; }
      if (!threw) throw new Error(msg || "Expected function to throw");
      if (match) {
        var s = String(err && (err.message || err));
        if (match instanceof RegExp) {
          if (!match.test(s)) throw new Error(msg || ("Expected error to match " + match + ", got: " + s));
        } else {
          if (s.indexOf(String(match)) === -1) throw new Error(msg || ("Expected error to contain \"" + match + "\", got: " + s));
        }
      }
    },
  };

  /**
   * Public API: harness.test(name, fn)
   * - fn can be sync or async (returning a Promise).
   * - Auto-finish when all tests settle, unless user calls harness.finish() manually first.
   */
  var running = 0;
  var finished = false;
  var failures = 0;
  var total = 0;
  var output = ensureOutputEl();

  /**
   * Begins a test case and records success/failure.
   * @param {string} name
   * @param {Function} fn  sync or async; may return a Promise
   */
  function test(name, fn) {
    if (finished) throw new Error("Cannot add tests after finish()");
    running++;
    total++;
    Promise.resolve()
      .then(fn)
      .then(function () {
        write("ok - " + name);
      })
      .catch(function (e) {
        failures++;
        write("not ok - " + name);
        // Include stack if present for easier debugging
        write(String(e && (e.stack || e)));
      })
      .finally(function () {
        running--;
        // Auto-finish when no tests are outstanding (unless caller wants manual control)
        if (running === 0 && !finished && window.__HARNESS_AUTO_FINISH__ !== false) {
          finish();
        }
      });
  }

  /**
   * Emits summary, stamps status in DOM, and prevents further tests.
   */
  function finish() {
    if (finished) return;
    finished = true;
    write("# tests: " + total);
    write("# passing: " + (total - failures));
    write("# failing: " + failures);
    setStatus(failures ? "failed" : "passed");
  }

  // Expose API
  window.harness = { test: test, assert: assert, finish: finish, log: write };
})();
