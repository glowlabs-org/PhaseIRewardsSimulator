(function () {
  "use strict";

  onReady(function () {
    harness.test("pager logic: state changes and callbacks", function () {
      let currentPage = 0;
      const onChange = (p) => { currentPage = p; };

      const pager = App.pager.create({
        totalItems: 100,
        pageSize: 10,
        currentPage: 0,
        windowSize: 5,
        onChange: onChange
      });
      
      const holder = document.createElement("div");
      document.body.appendChild(holder);
      holder.appendChild(pager);

      const getNext = () => qs(".icon-btn", pager).find(b => b.title.toLowerCase().includes("next"));
      const getPrev = () => qs(".icon-btn", pager).find(b => b.title.toLowerCase().includes("previous"));
      const findChip = (page) => qs(".page-chip", pager).find(b => text(b) === String(page));

      harness.assert.equal(currentPage, 0, "initial page is 0");
      harness.assert.truthy(getPrev().disabled, "prev is disabled on first page");

      click(getNext());
      harness.assert.equal(currentPage, 1, "next click increments page");
      harness.assert.truthy(!getPrev().disabled, "prev is enabled after moving");

      click(getPrev());
      harness.assert.equal(currentPage, 0, "prev click decrements page");

      let chip5 = findChip(5);
      harness.assert.truthy(chip5, "page 5 chip should be visible");
      click(chip5);
      harness.assert.equal(currentPage, 4, "clicking page 5 chip changes to page 4 (0-indexed)");

      // Test last page
      const totalPages = Math.ceil(100 / 10);
      const lastPageChip = findChip(totalPages);
      click(lastPageChip);
      harness.assert.equal(currentPage, totalPages - 1, "at last page");
      harness.assert.truthy(getNext().disabled, "next disabled on last page");

      holder.remove();
    });

    harness.test("pager logic: keyboard navigation", function () {
      let currentPage = 3;
      const onChange = (p) => { currentPage = p; };

      const pager = App.pager.create({
        totalItems: 100,
        pageSize: 10,
        currentPage: 3,
        windowSize: 5,
        onChange: onChange
      });

      const holder = document.createElement("div");
      document.body.appendChild(holder);
      holder.appendChild(pager);

      pager.focus();
      pager.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }));
      harness.assert.equal(currentPage, 4, "ArrowRight increments page");

      pager.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft", bubbles: true }));
      harness.assert.equal(currentPage, 3, "ArrowLeft decrements page");

      pager.dispatchEvent(new KeyboardEvent("keydown", { key: "End", bubbles: true }));
      harness.assert.equal(currentPage, 9, "End key goes to last page");

      pager.dispatchEvent(new KeyboardEvent("keydown", { key: "Home", bubbles: true }));
      harness.assert.equal(currentPage, 0, "Home key goes to first page");
      
      holder.remove();
    });
  });
})();