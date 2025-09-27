(function () {
  "use strict";

  const E = (sel, root = document) => root.querySelector(sel);
  const Es = (sel, root = document) => Array.from(root.querySelectorAll(sel));

  const SCALE_BI = 1000000000000000000n;

  // App state
  let farms = [];
  let nextId = 1;
  let diagnostics = null;

  function defaultFarm() {
    return {
      id: nextId++,
      firstWeek: 1,
      weeksAlive: 10,
      weeklyCC: 0.1,
      protocolDeposit: 50000,
      assetPrice: 0.4,
      edit: false,
    };
  }

  // ---- BigInt helpers for safe numeric handling ----
  function toBI(x) {
    if (x == null) return 0n;
    if (typeof x === "bigint") return x;
    if (typeof x === "string") {
      const s = x.trim();
      if (!s) return 0n;
      if (/^-?\d+$/.test(s)) return BigInt(s);
      const cleaned = s.replace(/[^0-9-]/g, "");
      if (cleaned === "" || cleaned === "-" || cleaned === "+") return 0n;
      try { return BigInt(cleaned); } catch { return 0n; }
    }
    if (typeof x === "number") {
      if (!Number.isFinite(x)) return 0n;
      return BigInt(Math.trunc(x));
    }
    if (Array.isArray(x)) {
      try {
        const sign = Number(x[0] || 0);
        const limbs = Array.isArray(x[1]) ? x[1] : [];
        const BASE64 = 2n ** 64n;
        let acc = 0n;
        for (let i = limbs.length - 1; i >= 0; i--) {
          const li = limbs[i];
          const v = typeof li === "number" ? BigInt(Math.trunc(li)) : BigInt(String(li).replace(/[^\d]/g, "") || "0");
          acc = acc * BASE64 + v;
        }
        return sign < 0 ? -acc : acc;
      } catch {
        try {
          const sign = Number(x[0] || 0);
          const limbs = Array.isArray(x[1]) ? x[1] : [];
          const BASE32 = 2n ** 32n;
          let acc = 0n;
          for (let i = limbs.length - 1; i >= 0; i--) {
            const li = limbs[i];
            const v = typeof li === "number" ? BigInt(Math.trunc(li)) : BigInt(String(li).replace(/[^\d]/g, "") || "0");
            acc = acc * BASE32 + v;
          }
          return sign < 0 ? -acc : acc;
        } catch {
          return 0n;
        }
      }
    }
    if (typeof x === "object") {
      if (typeof x.value === "string") return toBI(x.value);
      if (typeof x.data === "string") return toBI(x.data);
      return toBI(String(x));
    }
    return 0n;
  }

  function pow10BI(n) {
    return BigInt("1" + "0".repeat(Number(n)));
  }

  function addCommasToFormatted(str) {
    const s = String(str);
    const neg = s.startsWith("-");
    const [intPartRaw, frac = ""] = (neg ? s.slice(1) : s).split(".");
    const intPart = intPartRaw.replace(/^0+(?=\d)/, "");
    const withCommas = intPart.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
    return (neg ? "-" : "") + withCommas + (frac ? "." + frac : "");
  }

  function formatNumScaled(x, maxFrac = 6) {
    const bi = toBI(x);
    const neg = bi < 0n;
    const abs = neg ? -bi : bi;

    const intPart = abs / SCALE_BI;
    const fracFull = abs % SCALE_BI;

    if (maxFrac <= 0) {
      return (neg ? "-" : "") + intPart.toString();
    }
    const drop = 18 - Math.min(18, maxFrac);
    const fracTrimmed = drop > 0 ? (fracFull / pow10BI(drop)) : fracFull;
    if (fracTrimmed === 0n) {
      return (neg ? "-" : "") + intPart.toString();
    }
    let fracStr = fracTrimmed.toString().padStart(Math.min(18, maxFrac), "0");
    fracStr = fracStr.replace(/0+$/, "");
    const out = (neg ? "-" : "") + intPart.toString() + (fracStr ? "." + fracStr : "");
    return out;
  }

  function formatScaledWithCommas(x, maxFrac = 6) {
    return addCommasToFormatted(formatNumScaled(x, maxFrac));
  }

  function formatDollarsScaled(x) {
    return "$" + formatScaledWithCommas(x, 2);
  }

  function formatTokensScaled(x) {
    return formatScaledWithCommas(x, 6) + " GLW";
  }

  function formatPlainNumber(x, maxFrac = 6) {
    return Number(x).toLocaleString(undefined, { maximumFractionDigits: maxFrac });
  }

  function randomEthAddress() {
    const hex = [...crypto.getRandomValues(new Uint8Array(20))]
      .map(b => b.toString(16).padStart(2, "0"))
      .join("");
    return "0x" + hex;
  }

  function toScaledIntString(decStr, scaleDigits) {
    const s = String(decStr).trim();
    if (!s) return "0";
    const parts = s.split(".");
    const intPart = parts[0].replace(/[^\d]/g, "") || "0";
    const frac = (parts[1] || "").replace(/[^\d]/g, "");
    const fracPadded = (frac + "0".repeat(scaleDigits)).slice(0, scaleDigits);
    const out = (intPart + fracPadded).replace(/^0+/, "");
    return out.length ? out : "0";
  }

  function bigPow10(n) {
    return BigInt("1" + "0".repeat(n));
  }

  function buildApiInput() {
    const sorted = [...farms].sort((a, b) => (a.firstWeek - b.firstWeek) || (a.id - b.id));

    const solar_farms = sorted.map(f => {
      const wcc = toScaledIntString(f.weeklyCC, 18);
      const pd = toScaledIntString(f.protocolDeposit, 18);
      const ap = toScaledIntString(f.assetPrice.toFixed(2), 18);

      const pdBI = BigInt(pd);
      const apBI = BigInt(ap || "1"); // guard, though asset price input enforces >=0.01
      const arScaled = (pdBI * bigPow10(18)) / (apBI === 0n ? 1n : apBI);

      return {
        farm_id: String(f.id),
        asset_id: "glw",
        region_id: "simulation",
        weekly_carbon_credits: wcc,
        protocol_deposit_value: pd,
        assets_required: arScaled.toString(),
        rewards_address: randomEthAddress(),
        first_week: Number(f.firstWeek),
        weeks_alive: Math.max(2, Number(f.weeksAlive))
      };
    });

    return {
      cgp_leftovers: {},
      solar_farms
    };
  }

  // ----- Formatting helpers for designer (non-scaled numbers) -----
  function formatMoneyUSD(num) {
    const n = Number(num) || 0;
    return n.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0, style: "currency", currency: "USD" });
  }
  function formatPriceUSD2(num) {
    const n = Number(num) || 0;
    return n.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2, style: "currency", currency: "USD" });
  }

  function farmCardView(f) {
    const container = document.createElement("div");
    container.className = "card" + (f.edit ? " editing" : "");

    if (f.edit) {
      const header = document.createElement("div");
      header.className = "card-header";
      header.innerHTML = `<div class="card-title">Farm #${f.id} (edit)</div>`;
      container.appendChild(header);

      const form = document.createElement("div");
      form.className = "inline-form";
      form.innerHTML = `
        <label>First week<input type="number" min="1" value="${f.firstWeek}" data-key="firstWeek"></label>
        <label>Weeks alive<input type="number" min="2" value="${f.weeksAlive}" data-key="weeksAlive"></label>
        <label>Weekly CC<input type="number" step="0.000001" min="0.000000000000000001" value="${f.weeklyCC}" data-key="weeklyCC"></label>
        <label>Protocol deposit ($)<input type="number" step="0.01" min="0.01" value="${f.protocolDeposit}" data-key="protocolDeposit"></label>
        <label>GLW Price<input type="number" step="0.01" min="0.01" value="${Number(f.assetPrice).toFixed(2)}" data-key="assetPrice"></label>
      `;
      container.appendChild(form);

      const actions = document.createElement("div");
      actions.className = "card-actions";
      const btnSave = document.createElement("button");
      btnSave.className = "btn btn-primary";
      btnSave.textContent = "Save";
      btnSave.onclick = () => {
        Es("input", form).forEach(inp => {
          const key = inp.getAttribute("data-key");
          const val = inp.value;
          if (key === "firstWeek" || key === "weeksAlive") {
            f[key] = Math.max((key === "weeksAlive" ? 2 : 1), parseInt(val, 10) || 0);
          } else if (key === "assetPrice") {
            const v = parseFloat(val) || 0;
            f.assetPrice = Math.max(0.01, Math.round(v * 100) / 100);
          } else {
            f[key] = parseFloat(val) || 0;
          }
        });
        f.edit = false;
        renderDesigner();
      };

      const btnCancel = document.createElement("button");
      btnCancel.className = "btn btn-secondary";
      btnCancel.textContent = "Cancel";
      btnCancel.onclick = () => { f.edit = false; renderDesigner(); };

      actions.append(btnSave, btnCancel);
      container.appendChild(actions);
    } else {
      const header = document.createElement("div");
      header.className = "card-header";
      header.innerHTML = `<div class="card-title">Farm #${f.id}</div><div class="card-subtitle">Week ${f.firstWeek} • ${f.weeksAlive} weeks</div>`;
      container.appendChild(header);

      const kv = document.createElement("div");
      kv.className = "kv";
      kv.innerHTML = `
        <div>Weekly CC<br><strong>${formatPlainNumber(f.weeklyCC)}</strong></div>
        <div>Deposit<br><strong>${formatMoneyUSD(f.protocolDeposit)}</strong></div>
        <div>GLW Price<br><strong>${formatPriceUSD2(f.assetPrice)}</strong></div>
      `;
      container.appendChild(kv);

      const actions = document.createElement("div");
      actions.className = "card-actions";
      const btnEdit = document.createElement("button");
      btnEdit.className = "btn btn-ghost";
      btnEdit.textContent = "Edit";
      btnEdit.onclick = () => { f.edit = true; renderDesigner(); };

      const btnDel = document.createElement("button");
      btnDel.className = "btn btn-secondary";
      btnDel.textContent = "Delete";
      btnDel.onclick = () => {
        farms = farms.filter(x => x !== f);
        renderDesigner();
      };
      actions.append(btnEdit, btnDel);
      container.appendChild(actions);
    }
    return container;
  }

  function renderDesigner() {
    farms.sort((a,b)=>(a.firstWeek - b.firstWeek) || (a.id - b.id));
    const holder = E("#farmCards");
    holder.innerHTML = "";

    farms.forEach(f => holder.appendChild(farmCardView(f)));

    const add = document.createElement("div");
    add.className = "card add-card";
    add.textContent = "+ Add a farm";
    add.onclick = () => {
      farms.push(defaultFarm());
      renderDesigner();
    };
    holder.appendChild(add);
  }

  function setStatus(msg) {
    E("#status").textContent = msg || "";
  }

  async function simulate() {
    setStatus("Simulating...");
    diagnostics = null;
    E("#warnings").innerHTML = "";
    E("#weekCards").innerHTML = "";
    E("#weekDetails").innerHTML = "";
    E("#farmSummaryCards").innerHTML = "";
    E("#farmDetails").innerHTML = "";

    const body = buildApiInput();
    if (!body.solar_farms.length) {
      setStatus("Please add at least one farm.");
      return;
    }
    try {
      const res = await fetch("/api/rewards-simulator-detailed?as_strings=true", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body)
      });
      const data = await res.json();
      if (res.status === 200 || res.status === 422) {
        diagnostics = data;
        const errs = data.errors || [];
        if (errs.length) {
          const w = E("#warnings");
          w.innerHTML = `<div class="card" style="border-left:4px solid var(--orange)"><strong>Warnings:</strong><ul>${errs.map(e=>`<li>${escapeHtml(e)}</li>`).join("")}</ul></div>`;
        }
        renderPerWeek();
        renderPerFarm();
        setStatus("Simulation complete.");
      } else {
        setStatus("Simulation failed.");
        const w = E("#warnings");
        w.innerHTML = `<div class="card" style="border-left:4px solid var(--orange)"><strong>Error:</strong> ${escapeHtml(data.error || "Unknown error")}</div>`;
      }
    } catch (err) {
      setStatus("Network error.");
      const w = E("#warnings");
      w.innerHTML = `<div class="card" style="border-left:4px solid var(--orange)"><strong>Error:</strong> ${escapeHtml(String(err))}</div>`;
    }
  }

  function escapeHtml(s) {
    return String(s).replace(/[&<>'"]/g, c => ({'&':"&amp;",'<':"&lt;",'>':"&gt;","'":"&#39;",'"':"&quot;"}[c]));
  }

  // ----- Per-week view -----
  function renderPerWeek() {
    if (!diagnostics) return;
    const comps = diagnostics.competitions || [];
    const weeksMap = new Map();

    for (const comp of comps) {
      for (const b of comp.buckets) {
        const w = b.week_number;
        if (!weeksMap.has(w)) {
          weeksMap.set(w, {
            total_deposits: 0n,
            total_carbon: 0n,
            pool_assets: 0n,
            pool_deposits: 0n,
            participants: 0,
            actives: []
          });
        }
        const agg = weeksMap.get(w);
        agg.total_deposits += toBI(b.total_deposits);
        agg.total_carbon += toBI(b.total_carbon_credits);
        agg.pool_assets += toBI(b.pool_net_assets);
        agg.pool_deposits += toBI(b.pool_net_deposits);
        const states = Array.isArray(b.farm_states) ? b.farm_states : [];
        agg.participants += states.length;
        for (const st of states) {
          agg.actives.push({ comp, weekBucket: b, farm_id: st.farm_id });
        }
      }
    }

    const weekCards = E("#weekCards");
    weekCards.innerHTML = "";
    const sortedWeeks = Array.from(weeksMap.keys()).sort((a,b)=>a-b);

    for (const w of sortedWeeks) {
      const item = weeksMap.get(w);
      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Week ${w}</div>
          <div class="badge">${item.participants} farms</div>
        </div>
        <div class="kv">
          <div>Total deposits<br><strong>${formatDollarsScaled(item.total_deposits)}</strong></div>
          <div>Total carbon<br><strong>${formatScaledWithCommas(item.total_carbon, 6)}</strong></div>
          <div>Pool net assets<br><strong>${formatTokensScaled(item.pool_assets)}</strong></div>
          <div>Pool net deposits<br><strong>${formatDollarsScaled(item.pool_deposits)}</strong></div>
        </div>
      `;
      card.style.cursor = "pointer";
      card.onclick = () => renderWeekDetails(w, item);
      weekCards.appendChild(card);
    }
    if (sortedWeeks.length) renderWeekDetails(sortedWeeks[0], weeksMap.get(sortedWeeks[0]));
  }

  function renderWeekDetails(weekNumber, agg) {
    const details = E("#weekDetails");
    details.innerHTML = "";
    const items = (agg && agg.actives) ? agg.actives : [];
    if (!items.length) {
      const none = document.createElement("div");
      none.className = "card";
      none.textContent = "No farms active this week.";
      details.appendChild(none);
      return;
    }

    for (const j of items) {
      const { comp, weekBucket, farm_id } = j;
      const st = (weekBucket.farm_states || []).find(s => s.farm_id === farm_id);
      const finfo = (comp.farms || []).find(x => x.farm_id === farm_id);
      if (!st || !finfo) continue;

      const kind = weekNumber === finfo.first_week ? "first" : (weekNumber === finfo.final_week ? "last" : "ongoing");

      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Farm #${farm_id}</div>
          <span class="badge ${kind}">${kind}</span>
        </div>
        <div class="kv">
          <div>Deposits contributed<br><strong>${formatDollarsScaled(st.deposits_contributed)}</strong></div>
          <div>Carbon contributed<br><strong>${formatScaledWithCommas(st.carbon_credits_contributed, 6)}</strong></div>
          <div>Accum. drawdown<br><strong>${formatDollarsScaled(st.accumulated_drawdown)}</strong></div>
          <div>Net overperf.<br><strong>${formatDollarsScaled(st.net_overperformance)}</strong></div>
          <div>Rewards this week<br><strong>${formatTokensScaled(st.rewards_this_week)}</strong></div>
        </div>
      `;
      details.appendChild(card);
    }
  }

  // ----- Per-farm view -----
  function renderPerFarm() {
    if (!diagnostics) return;
    const comps = diagnostics.competitions || [];
    const farmMap = new Map();

    for (const comp of comps) {
      for (const b of comp.buckets) {
        for (const st of b.farm_states || []) {
          const fid = st.farm_id;
          if (!farmMap.has(fid)) {
            const finfo = (comp.farms || []).find(x => x.farm_id === fid) || {};
            farmMap.set(fid, { meta: { ...finfo }, entries: [], totalBI: 0n });
          }
          const rec = farmMap.get(fid);
          rec.entries.push({ comp, b, st, week: b.week_number });
          rec.totalBI += toBI(st.rewards_this_week);
        }
      }
    }

    const farmArr = Array.from(farmMap.entries()).map(([fid, v]) => {
      return { fid, totalBI: v.totalBI, ...v };
    }).sort((a,b)=>String(a.fid).localeCompare(String(b.fid)));

    const holder = E("#farmSummaryCards");
    holder.innerHTML = "";
    for (const f of farmArr) {
      const card = document.createElement("div");
      card.className = "card";
      card.style.cursor = "pointer";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Farm #${escapeHtml(f.fid)}</div>
        </div>
        <div class="kv">
          <div>Total rewards<br><strong>${formatTokensScaled(f.totalBI)}</strong></div>
          <div>Weeks<br><strong>${(f.entries||[]).length}</strong></div>
        </div>
      `;
      card.onclick = () => renderFarmDetails(f);
      holder.appendChild(card);
    }
    if (farmArr.length) renderFarmDetails(farmArr[0]);
  }

  function renderFarmDetails(farmObj) {
    const d = E("#farmDetails");
    d.innerHTML = "";
    const entries = (farmObj.entries || []).sort((a,b)=>a.week - b.week);

    for (const e of entries) {
      const b = e.b;
      const st = e.st;
      const td = toBI(b.total_deposits);
      const fcc = toBI(st.carbon_credits_contributed);
      const tc = toBI(b.total_carbon_credits) || 1n;
      const depRecBI = (td * fcc) / tc;

      const kind = e.week === farmObj.meta.first_week ? "first" : (e.week === farmObj.meta.final_week ? "last" : "ongoing");

      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Week ${e.week}</div>
          <span class="badge ${kind}">${kind}</span>
        </div>
        <div class="kv">
          <div>Total deposits<br><strong>${formatDollarsScaled(b.total_deposits)}</strong></div>
          <div>Farm deposits<br><strong>${formatDollarsScaled(st.deposits_contributed)}</strong></div>
          <div>Total carbon<br><strong>${formatScaledWithCommas(b.total_carbon_credits, 6)}</strong></div>
          <div>Farm carbon<br><strong>${formatScaledWithCommas(st.carbon_credits_contributed, 6)}</strong></div>
          <div>Deposits recovered<br><strong>${formatDollarsScaled(depRecBI)}</strong></div>
          <div>Pool net assets<br><strong>${formatTokensScaled(b.pool_net_assets)}</strong></div>
          <div>Pool net deposits<br><strong>${formatDollarsScaled(b.pool_net_deposits)}</strong></div>
          <div>Accum. drawdown<br><strong>${formatDollarsScaled(st.accumulated_drawdown)}</strong></div>
          <div>Net overperf.<br><strong>${formatDollarsScaled(st.net_overperformance)}</strong></div>
          <div>Rewards this week<br><strong>${formatTokensScaled(st.rewards_this_week)}</strong></div>
        </div>
      `;
      d.appendChild(card);
    }
  }

  // Tabs
  function setupTabs() {
    const tabWeek = E("#tabWeek");
    const tabFarm = E("#tabFarm");
    const perWeek = E("#perWeek");
    const perFarm = E("#perFarm");

    tabWeek.onclick = () => {
      tabWeek.classList.add("active");
      tabFarm.classList.remove("active");
      perWeek.classList.remove("hidden");
      perFarm.classList.add("hidden");
    };
    tabFarm.onclick = () => {
      tabFarm.classList.add("active");
      tabWeek.classList.remove("active");
      perFarm.classList.remove("hidden");
      perWeek.classList.add("hidden");
    };
  }

  function init() {
    setupTabs();
    E("#simulateBtn").onclick = simulate;
    farms.push(defaultFarm());
    renderDesigner();
  }

  window.addEventListener("DOMContentLoaded", init);
})();