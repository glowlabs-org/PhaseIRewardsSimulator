(function () {
  "use strict";

  onReady(function () {
    harness.test("pager layout: arrows same line, indicator centered on next line", function () {
      const pager = App.pager.create({
        totalItems: 100,
        pageSize: 5,
        currentPage: 0,
        windowSize: 7,
        onChange: function () {}
      });

      const holder = document.createElement("div");
      document.body.appendChild(holder);
      holder.appendChild(pager);

      harness.assert.truthy(pager.classList.contains("pagination"), "pager root should have .pagination");

      const row = pager.firstElementChild;
      const indicator = pager.lastElementChild;

      harness.assert.truthy(!!row && row.classList.contains("pagination-row"), "first child should be .pagination-row");
      harness.assert.truthy(!!indicator && indicator.classList.contains("page-indicator"), "last child should be .page-indicator");

      const prev = row.querySelectorAll(".icon-btn")[0];
      const next = row.querySelectorAll(".icon-btn")[1];
      const chips = row.querySelector(".page-chips");

      harness.assert.truthy(!!prev && !!next, "prev/next icon buttons should exist");
      harness.assert.truthy(!!chips, "page-chips should exist in row");

      const csRow = getComputedStyle(row);
      harness.assert.equal(csRow.flexWrap, "nowrap", "pagination-row should not wrap");

      const csRoot = getComputedStyle(pager);
      harness.assert.equal(csRoot.flexDirection, "column", "pagination root should be column");

      const text = String(indicator.textContent || "");
      harness.assert.truthy(text.toLowerCase().indexOf("page 1 of") !== -1, "indicator text should contain 'Page 1 of'");
    });
  });
})();