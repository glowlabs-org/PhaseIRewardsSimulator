(function () {
  "use strict";

  const E = (sel, root = document) => root.querySelector(sel);
  const Es = (sel, root = document) => Array.from(root.querySelectorAll(sel));

  const SCALE = 1e18; // numeric for inputs, but rendering uses bigint-safe formatter
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
      // Allow optional sign and digits only.
      if (/^-?\d+$/.test(s)) return BigInt(s);
      // Fallback: strip non-digits (keeps sign if present)
      const cleaned = s.replace(/[^0-9-]/g, "");
      if (cleaned === "" || cleaned === "-" || cleaned === "+") return 0n;
      try { return BigInt(cleaned); } catch { return 0n; }
    }
    if (typeof x === "number") {
      if (!Number.isFinite(x)) return 0n;
      // Avoid fractional parts
      return BigInt(Math.trunc(x));
    }
    if (typeof x === "object") {
      // Try common fields
      if (typeof x.value === "string") return toBI(x.value);
      if (typeof x.data === "string") return toBI(x.data);
      // Last resort: stringify and parse digits
      return toBI(String(x));
    }
    return 0n;
  }

  function pow10BI(n) {
    // n is small (<= 18) in our usage
    return BigInt("1" + "0".repeat(Number(n)));
  }

  function formatNumScaled(x, maxFrac = 6) {
    // x may be bigint/number/string representing a scaled integer (scale=1e18)
    // Render a human string with up to maxFrac fractional digits.
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
    // Remove trailing zeros
    fracStr = fracStr.replace(/0+$/, "");
    return (neg ? "-" : "") + intPart.toString() + "." + fracStr;
  }

  function formatPlain(x) {
    return Number(x).toLocaleString(undefined, { maximumFractionDigits: 6 });
  }

  function randomEthAddress() {
    const hex = [...crypto.getRandomValues(new Uint8Array(20))]
      .map(b => b.toString(16).padStart(2, "0"))
      .join("");
    return "0x" + hex;
  }

  function toScaledIntString(decStr, scaleDigits) {
    // Convert a decimal string like "0.4" to an integer string scaled by scaleDigits.
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
    // Sort farms: firstWeek asc, then id asc
    const sorted = [...farms].sort((a, b) => (a.firstWeek - b.firstWeek) || (a.id - b.id));

    const solar_farms = sorted.map(f => {
      const wcc = toScaledIntString(f.weeklyCC, 18);
      const pd = toScaledIntString(f.protocolDeposit, 18);
      const ap = toScaledIntString(f.assetPrice, 18);

      const pdBI = BigInt(pd);
      const apBI = BigInt(ap);
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

  function farmCardView(f) {
    const container = document.createElement("div");
    container.className = "card";

    if (f.edit) {
      // Edit mode
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
        <label>Asset price ($)<input type="number" step="0.000001" min="0.000000000000000001" value="${f.assetPrice}" data-key="assetPrice"></label>
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
            f[key] = Math.max( (key === "weeksAlive" ? 2 : 1), parseInt(val,10) || 0);
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
        <div>Weekly CC<br><strong>${formatPlain(f.weeklyCC)}</strong></div>
        <div>Deposit ($)<br><strong>${formatPlain(f.protocolDeposit)}</strong></div>
        <div>Asset Price ($)<br><strong>${formatPlain(f.assetPrice)}</strong></div>
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
    // Sort display
    farms.sort((a,b)=>(a.firstWeek - b.firstWeek) || (a.id - b.id));
    const holder = E("#farmCards");
    holder.innerHTML = "";

    farms.forEach(f => holder.appendChild(farmCardView(f)));

    // Add card
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
      const res = await fetch("/api/rewards-simulator-detailed", {
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
    // Build an index of weeks aggregated across competitions
    const weeksMap = new Map(); // week -> { total_deposits(BigInt), total_carbon(BigInt), pool_assets(BigInt), pool_deposits(BigInt), participants, joiners: [] }

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
            joiners: []
          });
        }
        const agg = weeksMap.get(w);
        agg.total_deposits += toBI(b.total_deposits);
        agg.total_carbon += toBI(b.total_carbon_credits);
        agg.pool_assets += toBI(b.pool_net_assets);
        agg.pool_deposits += toBI(b.pool_net_deposits);
        agg.participants += (b.farm_states || []).length;
        for (const fid of (b.first_week_farms || [])) {
          agg.joiners.push({ comp, weekBucket: b, farm_id: fid });
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
          <div>Total deposits<br><strong>${formatNumScaled(item.total_deposits)}</strong></div>
          <div>Total carbon<br><strong>${formatNumScaled(item.total_carbon)}</strong></div>
          <div>Pool net assets<br><strong>${formatNumScaled(item.pool_assets)}</strong></div>
          <div>Pool net deposits<br><strong>${formatNumScaled(item.pool_deposits)}</strong></div>
        </div>
      `;
      card.style.cursor = "pointer";
      card.onclick = () => renderWeekDetails(w, item);
      weekCards.appendChild(card);
    }
    // auto-open first week
    if (sortedWeeks.length) renderWeekDetails(sortedWeeks[0], weeksMap.get(sortedWeeks[0]));
  }

  function renderWeekDetails(weekNumber, agg) {
    const details = E("#weekDetails");
    details.innerHTML = "";
    if (!agg || !agg.joiners || !agg.joiners.length) {
      const none = document.createElement("div");
      none.className = "card";
      none.textContent = "No farms started this week.";
      details.appendChild(none);
      return;
    }

    for (const j of agg.joiners) {
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
          <div>Deposits contributed<br><strong>${formatNumScaled(st.deposits_contributed)}</strong></div>
          <div>Carbon contributed<br><strong>${formatNumScaled(st.carbon_credits_contributed)}</strong></div>
          <div>Accum. drawdown<br><strong>${formatNumScaled(st.accumulated_drawdown)}</strong></div>
          <div>Net overperf.<br><strong>${formatNumScaled(st.net_overperformance)}</strong></div>
          <div>Rewards this week<br><strong>${formatNumScaled(st.rewards_this_week)}</strong></div>
        </div>
      `;
      details.appendChild(card);
    }
  }

  // ----- Per-farm view -----
  function renderPerFarm() {
    if (!diagnostics) return;
    const comps = diagnostics.competitions || [];
    const farmMap = new Map(); // id -> { meta, entries: [{comp, b, st, week}], totalBI }

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

    // compute totals and sort by id
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
          <div>Total rewards<br><strong>${formatNumScaled(f.totalBI)}</strong></div>
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
      // deposits recovered (frontend compute): total_deposits * carbon_credits_contributed / total_carbon_credits
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
          <div>Total deposits<br><strong>${formatNumScaled(b.total_deposits)}</strong></div>
          <div>Farm deposits<br><strong>${formatNumScaled(st.deposits_contributed)}</strong></div>
          <div>Total carbon<br><strong>${formatNumScaled(b.total_carbon_credits)}</strong></div>
          <div>Farm carbon<br><strong>${formatNumScaled(st.carbon_credits_contributed)}</strong></div>
          <div>Deposits recovered<br><strong>${formatNumScaled(depRecBI)}</strong></div>
          <div>Pool net assets<br><strong>${formatNumScaled(b.pool_net_assets)}</strong></div>
          <div>Pool net deposits<br><strong>${formatNumScaled(b.pool_net_deposits)}</strong></div>
          <div>Accum. drawdown<br><strong>${formatNumScaled(st.accumulated_drawdown)}</strong></div>
          <div>Net overperf.<br><strong>${formatNumScaled(st.net_overperformance)}</strong></div>
          <div>Rewards this week<br><strong>${formatNumScaled(st.rewards_this_week)}</strong></div>
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
    // Seed with one default farm and an always-present add card UX
    farms.push(defaultFarm());
    renderDesigner();
  }

  // Kick off
  window.addEventListener("DOMContentLoaded", init);
})();