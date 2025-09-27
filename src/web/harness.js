/**
 * Minimal DOM-based test harness (no deps).
 *
 * Key behaviors:
 *  - Writes ONLY to a hidden <pre id="__TEST_OUTPUT__"> sink.
 *  - Emits one standardized line per test:
 *      TEST name="<name>" status=passed|failed duration_ms=<float>
 *  - On failure, also prints the error/stack on following lines.
 *  - Stamps <html data-test-status="passed|failed"> for CI exit parsing.
 *  - Auto-finishes when all tests settle (unless __HARNESS_AUTO_FINISH__ === false).
 *
 * Public API (global):
 *   harness.test(name, fn)        // fn may be sync or async
 *   harness.assert.{equal,truthy,throws}
 *   harness.finish()              // optional manual finish
 *   harness.log(line)             // write an extra line to the sink
 */
(function () {
  "use strict";

  /**
   * Return a high-resolution timestamp if available, otherwise Date.now().
   * @returns {number}
   */
  function timeNow() {
    return (self.performance && typeof self.performance.now === "function")
      ? self.performance.now()
      : Date.now();
  }

  /**
   * Format milliseconds as a concise float (rounded to 2 decimals).
   * @param {number} ms
   * @returns {string}
   */
  function formatMs(ms) {
    var v = Math.round(ms * 100) / 100;
    // Avoid trailing ".00" verbosity: keep as plain JS number string
    return String(v);
  }

  /**
   * Ensure the hidden pre element exists and return it.
   * @returns {HTMLElement}
   */
function ensureOutputEl() {
  var existing = document.getElementById("__TEST_OUTPUT__");
  if (existing) return existing;

  var el = document.createElement("pre");
  el.id = "__TEST_OUTPUT__";
  el.style.whiteSpace = "pre-wrap";
  el.style.display = "none";

  // Attach immediately—<html> is present even before <body> is parsed.
  document.documentElement.appendChild(el);
  return el;
}

  var output = ensureOutputEl();

  /**
   * Append a single line to the harness-only output buffer.
   * @param {string} line
   */
  function write(line) {
    output.textContent += line + "\n";
  }

  /**
   * Stamp the final status into the <html> tag for easy parsing.
   * @param {"passed"|"failed"} status
   */
  function setStatus(status) {
    document.documentElement.setAttribute("data-test-status", status);
    try {
      document.title = "[TEST-" + status.toUpperCase() + "] " + (document.title || "");
    } catch (_) { /* ignore */ }
  }

  // ----- Assertions ----------------------------------------------------------

  var assert = {
    /**
     * Strict equality assertion.
     * @param {*} a
     * @param {*} b
     * @param {string=} msg
     */
    equal: function (a, b, msg) {
      if (a !== b) {
        throw new Error(msg || ("Expected ===\n  left: " + String(a) + "\n right: " + String(b)));
      }
    },

    /**
     * Truthiness assertion.
     * @param {*} x
     * @param {string=} msg
     */
    truthy: function (x, msg) {
      if (!x) {
        throw new Error(msg || ("Expected truthy, got: " + String(x)));
      }
    },

    /**
     * Assert that a function throws; optionally match message.
     * @param {Function} fn
     * @param {RegExp|string=} match
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

  // ----- Runner core ---------------------------------------------------------

  var running = 0;
  var finished = false;
  var failures = 0;
  var total = 0;

  /**
   * Register and execute a test. fn may be sync or async.
   * Emits a single standardized line per test:
   *   TEST name="<name>" status=passed|failed duration_ms=<float>
   * On failure also prints the error/stack on subsequent lines.
   * @param {string} name
   * @param {Function} fn
   */
  function test(name, fn) {
    if (finished) throw new Error("Cannot add tests after finish()");
    running++;
    total++;

    var t0 = timeNow();
    var status = "passed";
    var err = null;

    Promise.resolve()
      .then(fn)
      .catch(function (e) {
        status = "failed";
        failures++;
        err = e;
      })
      .finally(function () {
        var dt = timeNow() - t0;
        // Single mandated per-test line:
        write('TEST name="' + String(name).replace(/"/g, '\\"') + '" status=' + status + ' duration_ms=' + formatMs(dt));
        // If failed, include stack/details on following line(s) for debugging:
        if (err) write(String(err && (err.stack || err)));

        running--;
        if (running === 0 && !finished && window.__HARNESS_AUTO_FINISH__ !== false) {
          finish();
        }
      });
  }

  /**
   * Emit a summary and set final status. Safe to call multiple times.
   */
  function finish() {
    if (finished) return;
    finished = true;
    write("# tests: " + total);
    write("# passing: " + (total - failures));
    write("# failing: " + failures);
    setStatus(failures ? "failed" : "passed");
  }

  // ----- Public API ----------------------------------------------------------

  // Expose a small API on window.
  window.harness = {
    test: test,
    assert: assert,
    finish: finish,
    /**
     * Write an extra line to the output sink.
     * @param {string} line
     */
    log: write
  };
})();
