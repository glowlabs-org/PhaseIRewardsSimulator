(function(){
  "use strict";
  // Prevent auto-finish while test chunks are loading; we'll finish when idle.
  try { window.__HARNESS_AUTO_FINISH__ = false; } catch (_) {}

  var v = "v3";
  var parts = [
    "/js/tests/helpers.js?" + v,
    "/js/tests/test_ma_e2e.js?" + v
  ];

  function loadSeq(i){
    if (i >= parts.length) {
      // After all test modules are loaded and have registered tests,
      // finish once all running tests have completed.
      if (window.harness && typeof window.harness.whenIdle === "function") {
        window.harness.whenIdle(function () {
          // allow any final microtasks to queue additional work
          setTimeout(function () {
            if (window.harness) window.harness.finish();
          }, 0);
        });
      } else {
        // Fallback: if harness isn't present for some reason, do nothing.
      }
      return;
    }
    var s = document.createElement("script");
    s.src = parts[i];
    s.onload = function(){ loadSeq(i+1); };
    s.onerror = function(){ loadSeq(i+1); };
    document.head.appendChild(s);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function(){ loadSeq(0); });
  } else {
    loadSeq(0);
  }
})();