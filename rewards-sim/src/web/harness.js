(function () {
  "use strict";

  function timeNow() {
    return (self.performance && typeof self.performance.now === "function")
      ? self.performance.now()
      : Date.now();
  }

  function formatMs(ms) {
    var v = Math.round(ms * 100) / 100;
    return String(v);
  }

  function ensureOutputEl() {
    var existing = document.getElementById("__TEST_OUTPUT__");
    if (existing) return existing;

    var el = document.createElement("pre");
    el.id = "__TEST_OUTPUT__";
    el.style.whiteSpace = "pre-wrap";
    el.style.display = "none";

    document.documentElement.appendChild(el);
    return el;
  }

  var output = ensureOutputEl();

  function write(line) {
    output.textContent += line + "\n";
  }

  function setStatus(status) {
    document.documentElement.setAttribute("data-test-status", status);
    try {
      document.title = "[TEST-" + status.toUpperCase() + "] " + (document.title || "");
    } catch (_) { /* ignore */ }
  }

  var assert = {
    equal: function (a, b, msg) {
      if (a !== b) {
        throw new Error(msg || ("Expected ===\n  left: " + String(a) + "\n right: " + String(b)));
      }
    },

    truthy: function (x, msg) {
      if (!x) {
        throw new Error(msg || ("Expected truthy, got: " + String(x)));
      }
    },

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

  var running = 0;
  var finished = false;
  var failures = 0;
  var total = 0;
  var idleListeners = [];

  function drainIdleListeners() {
    if (running === 0) {
      var toRun = idleListeners.slice();
      idleListeners.length = 0;
      toRun.forEach(function (cb) {
        try { cb(); } catch (_) { /* ignore listener errors */ }
      });
    }
  }

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
        write('TEST name="' + String(name).replace(/"/g, '\\"') + '" status=' + status + ' duration_ms=' + formatMs(dt));
        if (err) write(String(err && (err.stack || err)));

        running--;
        if (running === 0) {
          // Give any external controller a chance to react first.
          drainIdleListeners();
          if (!finished && window.__HARNESS_AUTO_FINISH__ !== false) {
            // Debounce finish slightly to allow late test registration from subsequently loaded scripts.
            var delay = typeof window.__HARNESS_FINISH_DEBOUNCE_MS__ === "number" ? window.__HARNESS_FINISH_DEBOUNCE_MS__ : 50;
            setTimeout(function () {
              if (running === 0 && !finished && window.__HARNESS_AUTO_FINISH__ !== false) {
                finish();
              }
            }, delay);
          }
        }
      });
  }

  function finish() {
    if (finished) return;
    finished = true;
    write("# tests: " + total);
    write("# passing: " + (total - failures));
    write("# failing: " + failures);
    setStatus(failures ? "failed" : "passed");
  }

  function whenIdle(cb) {
    if (typeof cb !== "function") return;
    if (running === 0) {
      // execute soon but asynchronously
      setTimeout(cb, 0);
    } else {
      idleListeners.push(cb);
    }
  }

  window.harness = {
    test: test,
    assert: assert,
    finish: finish,
    log: write,
    whenIdle: whenIdle
  };
})();