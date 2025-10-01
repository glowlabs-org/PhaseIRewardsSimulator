(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util || {};

  function clamp(v, lo, hi) {
    return Math.max(lo, Math.min(hi, v));
  }

  function buildRange(totalPages, current, windowSize) {
    const last = totalPages - 1;

    // A full pager can show: first, "...", window, "...", last.
    // A safe threshold to just show all pages is when total pages is
    // less than window size + room for first/last and ellipses.
    if (totalPages <= windowSize + 2) {
      const items = [];
      for (let i = 0; i < totalPages; i++) items.push(i);
      return items;
    }

    const w = Math.max(3, windowSize | 0);
    const half = Math.floor(w / 2);

    let start = current - half;
    let end = current + half;

    // Adjust window to stay within bounds [1, last-1]
    if (start < 1) {
      end = Math.min(end + (1 - start), last - 1);
      start = 1;
    }
    if (end > last - 1) {
      start = Math.max(start - (end - (last - 1)), 1);
      end = last - 1;
    }

    const items = [];
    items.push(0);
    if (start > 1) {
      if (start === 2) {
        items.push(1);
      } else {
        items.push("…");
      }
    }
    for (let i = start; i <= end; i++) items.push(i);
    if (end < last - 1) {
      if (end === last - 2) {
        items.push(last - 1);
      } else {
        items.push("…");
      }
    }
    if (last !== 0) {
      if (!items.includes(last)) {
        items.push(last);
      }
    }
    return items;
  }

  function create(opts) {
    const totalItems = Math.max(0, Number(opts.totalItems || 0));
    const pageSize = Math.max(1, Number(opts.pageSize || 1));
    const onChange = typeof opts.onChange === "function" ? opts.onChange : function () {};
    const windowSize = Number.isFinite(opts.windowSize) ? Math.max(3, Number(opts.windowSize)) : 5;

    const totalPages = Math.max(1, Math.ceil(totalItems / pageSize));
    let currentPage = clamp(Number(opts.currentPage || 0), 0, totalPages - 1);

    if (totalPages <= 1) {
      const frag = document.createElement("div");
      frag.className = "pagination";
      frag.setAttribute("aria-hidden", "true");
      return frag;
    }

    const root = document.createElement("nav");
    root.className = "pagination";
    root.setAttribute("role", "navigation");
    root.setAttribute("aria-label", "Pagination");
    root.tabIndex = 0;

    function render() {
      root.innerHTML = "";

      // Row container ensures prev/chips/next remain on the same line
      const row = document.createElement("div");
      row.className = "pagination-row";

      const prev = document.createElement("button");
      prev.type = "button";
      prev.className = "icon-btn";
      prev.title = "Previous page";
      prev.setAttribute("aria-label", "Previous page");
      prev.innerHTML = `
        <svg width="16" height="16" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="12 4 6 10 12 16"></polyline>
        </svg>`;
      prev.disabled = currentPage <= 0;
      prev.onclick = () => changeTo(currentPage - 1);
      row.appendChild(prev);

      const chipsWrap = document.createElement("div");
      chipsWrap.className = "page-chips";
      const range = buildRange(totalPages, currentPage, windowSize);
      for (const it of range) {
        if (it === "…") {
          const el = document.createElement("span");
          el.className = "ellipsis";
          el.textContent = "…";
          chipsWrap.appendChild(el);
          continue;
        }
        const idx = Number(it);
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "page-chip" + (idx === currentPage ? " active" : "");
        btn.textContent = String(idx + 1);
        btn.setAttribute("aria-current", idx === currentPage ? "page" : "false");
        btn.onclick = () => changeTo(idx);
        chipsWrap.appendChild(btn);
      }
      row.appendChild(chipsWrap);

      const next = document.createElement("button");
      next.type = "button";
      next.className = "icon-btn";
      next.title = "Next page";
      next.setAttribute("aria-label", "Next page");
      next.innerHTML = `
        <svg width="16" height="16" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="8 4 14 10 8 16"></polyline>
        </svg>`;
      next.disabled = currentPage >= (totalPages - 1);
      next.onclick = () => changeTo(currentPage + 1);
      row.appendChild(next);

      root.appendChild(row);

      // Indicator on its own centered line
      const ind = document.createElement("div");
      ind.className = "page-indicator";
      ind.textContent = "Page " + (currentPage + 1) + " of " + totalPages;
      root.appendChild(ind);
    }

    function changeTo(idx) {
      const clamped = clamp(idx, 0, totalPages - 1);
      if (clamped === currentPage) return;
      currentPage = clamped;
      render();
      try { onChange(currentPage); } catch (_) {}
    }

    root.addEventListener("keydown", (e) => {
      if (e.key === "ArrowLeft") { e.preventDefault(); changeTo(currentPage - 1); }
      if (e.key === "ArrowRight") { e.preventDefault(); changeTo(currentPage + 1); }
      if (e.key === "Home") { e.preventDefault(); changeTo(0); }
      if (e.key === "End") { e.preventDefault(); changeTo(Math.max(0, Math.ceil(totalItems / pageSize) - 1)); }
    });

    render();
    return root;
  }

  App.pager = { create };
})();