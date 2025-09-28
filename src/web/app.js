(function () {
  "use strict";

  const E = (sel, root = document) => root.querySelector(sel);
  const Es = (sel, root = document) => Array.from(root.querySelectorAll(sel));

  const SCALE_BI = 1000000000000000000n;

  let farms = [];
  let nextId = 1;
  let diagnostics = null;
  let addMode = false;

  function initialFarms() {
    // Per spec: 3 farms by default
    const items = [
      { id: nextId++, firstWeek: 1, weeksAlive: 5, weeklyCC: 0.08, protocolDeposit: 40000, assetPrice: 0.30, edit: false },
      { id: nextId++, firstWeek: 2, weeksAlive: 5, weeklyCC: 0.10, protocolDeposit: 80000, assetPrice: 0.40, edit: false },
      { id: nextId++, firstWeek: 2, weeksAlive: 5, weeklyCC: 0.12, protocolDeposit: 50000, assetPrice: 0.40, edit: false },
    ];
    return items;
  }

  function defaultFarm() {
    return {
      id: nextId++,
      firstWeek: 1,
      weeksAlive: 5,
      weeklyCC: 0.08,
      protocolDeposit: 40000,
      assetPrice: 0.30,
      edit: false,
    };
  }

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
        return sign < 0 ? -acc : (sign === 0 ? 0n : acc);
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
          return sign < 0 ? -acc : (sign === 0 ? 0n : acc);
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
    // Preserve current order; do not sort unless user clicks sort
    const solar_farms = farms.map(f => {
      const wcc = toScaledIntString(f.weeklyCC, 18);
      const pd = toScaledIntString(f.protocolDeposit, 18);
      const ap = toScaledIntString(f.assetPrice.toFixed(2), 18);

      const pdBI = BigInt(pd);
      const apBI = BigInt(ap || "1");
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

  function renderAddCard(parent) {
    const add = document.createElement("div");
    add.className = "card add-card";
    if (!addMode) {
      add.textContent = "+ Add a farm";
      add.onclick = () => { addMode = true; renderDesigner(); };
      parent.appendChild(add);
      return;
    }

    const f = defaultFarm();
    // add form UI
    const title = document.createElement("div");
    title.className = "card-header";
    title.innerHTML = `<div class="card-title">Add a farm</div>`;
    add.appendChild(title);

    const form = document.createElement("div");
    form.className = "inline-form";
    form.innerHTML = `
      <label>Farm ID<input type="number" min="1" value="${f.id}" data-key="id"></label>
      <label>First week<input type="number" min="1" value="${f.firstWeek}" data-key="firstWeek"></label>
      <label>Weeks alive<input type="number" min="2" value="${f.weeksAlive}" data-key="weeksAlive"></label>
      <label>Weekly CC<input type="number" step="0.000001" min="0.000000000000000001" value="${f.weeklyCC}" data-key="weeklyCC"></label>
      <label>Protocol deposit ($)<input type="number" step="0.01" min="0.01" value="${f.protocolDeposit}" data-key="protocolDeposit"></label>
      <label>GLW Price<input type="number" step="0.01" min="0.01" value="${Number(f.assetPrice).toFixed(2)}" data-key="assetPrice"></label>
    `;
    add.appendChild(form);

    const actions = document.createElement("div");
    actions.className = "card-actions";
    const submit = document.createElement("button");
    submit.className = "btn btn-primary";
    submit.textContent = "Submit";
    submit.onclick = () => {
      const obj = { ...f };
      Es("input", form).forEach(inp => {
        const key = inp.getAttribute("data-key");
        const val = inp.value;
        if (key === "id") {
          const nid = Math.max(1, parseInt(val, 10) || f.id);
          obj.id = nid;
        } else if (key === "firstWeek" || key === "weeksAlive") {
          obj[key] = Math.max((key === "weeksAlive" ? 2 : 1), parseInt(val, 10) || 0);
        } else if (key === "assetPrice") {
          const v = parseFloat(val) || 0;
          obj.assetPrice = Math.max(0.01, Math.round(v * 100) / 100);
        } else {
          obj[key] = parseFloat(val) || 0;
        }
      });
      // ensure unique id
      if (farms.some(x => String(x.id) === String(obj.id))) {
        setStatus("Farm ID already exists.");
        return;
      }
      // ensure nextId is > chosen id
      nextId = Math.max(nextId, Number(obj.id) + 1);
      obj.edit = false;
      farms.push(obj);
      addMode = false;
      renderDesigner();
    };

    const cancel = document.createElement("button");
    cancel.className = "btn btn-secondary";
    cancel.textContent = "Cancel";
    cancel.onclick = () => { addMode = false; renderDesigner(); };

    actions.append(submit, cancel);
    add.appendChild(actions);

    parent.appendChild(add);
  }

  function renderDesigner() {
    const holder = E("#farmCards");
    holder.innerHTML = "";

    farms.forEach(f => holder.appendChild(farmCardView(f)));
    renderAddCard(holder);
  }

  function setStatus(msg) {
    E("#status").textContent = msg || "";
  }

  async function simulate() {
    setStatus("Simulating...");
    diagnostics = null;
    E("#warnings").innerHTML = "";
    E("#weekCards").innerHTML = "";
    E("#weekHeadline").innerHTML = "";
    E("#weekDetails").innerHTML = "";
    E("#farmSummaryCards").innerHTML = "";
    E("#farmHeadline").innerHTML = "";
    E("#farmDetails").innerHTML = "";

    const body = buildApiInput();
    if (!body.solar_farms.length) {
      setStatus("Please add at least one farm.");
      return;
    }
    try {
      const res = await fetch("/api/rewards-simulator-detailed?bigints_as_strings=true", {
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

  function computeDepositsRecovered(totalDepositsBI, farmCCBI, totalCCBI) {
    const td = toBI(totalDepositsBI);
    const fcc = toBI(farmCCBI);
    const tcc = toBI(totalCCBI) || 1n;
    return (td * fcc) / tcc;
    }

  function findPrevNetOver(comp, week, farm_id) {
    const prevWeek = Number(week) - 1;
    const prevBucket = (comp.buckets || []).find(b => Number(b.week_number) === prevWeek);
    if (!prevBucket) return 0n;
    const st = (prevBucket.farm_states || []).find(s => String(s.farm_id) === String(farm_id));
    if (!st) return 0n;
    return toBI(st.net_overperformance);
  }

  function computePoolAndOwnGLW(comp, bucket, st, depRecBI) {
    const depositsContrib = toBI(st.deposits_contributed);
    const curNetOver = toBI(st.net_overperformance);
    const prevNetOver = findPrevNetOver(comp, bucket.week_number, st.farm_id);
    let baseOver = 0n;
    if (depRecBI > depositsContrib) {
      baseOver += depRecBI - depositsContrib;
    }
    baseOver += prevNetOver;
    baseOver -= curNetOver;
    if (baseOver < 0n) baseOver = 0n;

    const poolNetAssets = toBI(bucket.pool_net_assets);
    const poolNetDeposits = toBI(bucket.pool_net_deposits);
    let glwFromPool = 0n;
    if (poolNetDeposits > 0n) {
      glwFromPool = (baseOver * poolNetAssets) / poolNetDeposits;
      if (glwFromPool < 0n) glwFromPool = 0n;
    }
    const weekRewards = toBI(st.rewards_this_week);
    let glwFromOwn = weekRewards - glwFromPool;
    if (glwFromOwn < 0n) glwFromOwn = 0n;
    return { glwFromPool, glwFromOwn };
  }

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
            items: []
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
          agg.items.push({ comp, bucket: b, st });
        }
      }
    }

    const weekCards = E("#weekCards");
    weekCards.innerHTML = "";
    const sortedWeeks = Array.from(weeksMap.keys()).sort((a,b)=>a-b);

    for (const w of sortedWeeks) {
      const item = weeksMap.get(w);
      const card = document.createElement("div");
      card.className = "card compact";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Week ${w}</div>
          <div class="badge">${item.participants} farms</div>
        </div>
      `;
      card.style.cursor = "pointer";
      card.onclick = () => renderWeekDetails(w, item);
      weekCards.appendChild(card);
    }
    if (sortedWeeks.length) renderWeekDetails(sortedWeeks[0], weeksMap.get(sortedWeeks[0]));
  }

  function renderWeekHeadline(weekNumber, agg) {
    const head = E("#weekHeadline");
    head.innerHTML = "";
    const wrap = document.createElement("div");
    wrap.className = "card highlight";
    wrap.innerHTML = `
      <div class="card-header">
        <div class="card-title">Week ${weekNumber} Overview</div>
      </div>
      <div class="kv">
        <div>Total deposits<br><strong>${formatDollarsScaled(agg.total_deposits)}</strong></div>
        <div>Total carbon<br><strong>${formatScaledWithCommas(agg.total_carbon, 6)}</strong></div>
        <div>Farms<br><strong>${agg.participants}</strong></div>
        <div>Pool net assets<br><strong>${formatTokensScaled(agg.pool_assets)}</strong></div>
        <div>Pool net deposits<br><strong>${formatDollarsScaled(agg.pool_deposits)}</strong></div>
      </div>
    `;
    head.appendChild(wrap);
  }

  function renderWeekDetails(weekNumber, agg) {
    renderWeekHeadline(weekNumber, agg);
    const details = E("#weekDetails");
    details.innerHTML = "";

    const items = (agg && agg.items) ? agg.items : [];
    if (!items.length) {
      const none = document.createElement("div");
      none.className = "card";
      none.textContent = "No farms active this week.";
      details.appendChild(none);
      return;
    }

    for (const it of items) {
      const { comp, bucket, st } = it;
      const finfo = (comp.farms || []).find(x => x.farm_id === st.farm_id);
      const kind = (weekNumber === finfo.first_week) ? "first" : (weekNumber === finfo.final_week ? "last" : "ongoing");

      const depRec = computeDepositsRecovered(bucket.total_deposits, st.carbon_credits_contributed, bucket.total_carbon_credits);
      const parts = computePoolAndOwnGLW(comp, bucket, st, depRec);

      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Farm #${st.farm_id}</div>
          <span class="badge ${kind}">${kind}</span>
        </div>
        <div class="kv">
          <div>Deposits contributed<br><strong>${formatDollarsScaled(st.deposits_contributed)}</strong></div>
          <div>Carbon contributed<br><strong>${formatScaledWithCommas(st.carbon_credits_contributed, 6)}</strong></div>
          <div>Accum. drawdown<br><strong>${formatDollarsScaled(st.accumulated_drawdown)}</strong></div>
          <div>Net overperf.<br><strong>${formatDollarsScaled(st.net_overperformance)}</strong></div>
          <div>Deposits recovered<br><strong>${formatDollarsScaled(depRec)}</strong></div>
          <div>Rewards this week<br><strong>${formatTokensScaled(st.rewards_this_week)}</strong></div>
          <div>From own vault<br><strong>${formatTokensScaled(parts.glwFromOwn)}</strong></div>
          <div>From pool<br><strong>${formatTokensScaled(parts.glwFromPool)}</strong></div>
        </div>
      `;
      details.appendChild(card);
    }
  }

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
            farmMap.set(fid, { meta: { ...finfo }, entries: [] });
          }
          const rec = farmMap.get(fid);
          rec.entries.push({ comp, b, st, week: b.week_number });
        }
      }
    }

    const farmArr = Array.from(farmMap.entries()).map(([fid, v]) => ({ fid, ...v }))
      .sort((a,b)=>String(a.fid).localeCompare(String(b.fid)));

    const holder = E("#farmSummaryCards");
    holder.innerHTML = "";
    for (const f of farmArr) {
      const card = document.createElement("div");
      card.className = "card compact";
      card.style.cursor = "pointer";
      const deposit = f.meta && f.meta.protocol_deposit_value ? formatDollarsScaled(f.meta.protocol_deposit_value) : "$0";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Farm #${escapeHtml(f.fid)}</div>
        </div>
        <div class="kv">
          <div>Deposit<br><strong>${deposit}</strong></div>
        </div>
      `;
      card.onclick = () => renderFarmDetails(f);
      holder.appendChild(card);
    }
    if (farmArr.length) renderFarmDetails(farmArr[0]);
  }

  function renderFarmHeadline(farmObj) {
    const h = E("#farmHeadline");
    h.innerHTML = "";
    const m = farmObj.meta || {};
    const card = document.createElement("div");
    card.className = "card highlight";
    card.innerHTML = `
      <div class="card-header">
        <div class="card-title">Farm #${escapeHtml(farmObj.fid)} Overview</div>
      </div>
      <div class="kv">
        <div>Total deposit<br><strong>${formatDollarsScaled(m.protocol_deposit_value || 0)}</strong></div>
        <div>Assets required<br><strong>${formatTokensScaled(m.assets_required || 0)}</strong></div>
        <div>First week<br><strong>${Number(m.first_week || 0)}</strong></div>
        <div>Final week<br><strong>${Number(m.final_week || 0)}</strong></div>
      </div>
    `;
    h.appendChild(card);
  }

  function renderFarmDetails(farmObj) {
    renderFarmHeadline(farmObj);
    const d = E("#farmDetails");
    d.innerHTML = "";
    const entries = (farmObj.entries || []).sort((a,b)=>a.week - b.week);

    for (const e of entries) {
      const b = e.b;
      const st = e.st;
      const depRecBI = computeDepositsRecovered(b.total_deposits, st.carbon_credits_contributed, b.total_carbon_credits);
      const parts = computePoolAndOwnGLW(e.comp, b, st, depRecBI);

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
          <div>From own vault<br><strong>${formatTokensScaled(parts.glwFromOwn)}</strong></div>
          <div>From pool<br><strong>${formatTokensScaled(parts.glwFromPool)}</strong></div>
        </div>
      `;
      d.appendChild(card);
    }
  }

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

  function setupDesignerActions() {
    const sortBtn = E("#sortBtn");
    if (sortBtn) {
      sortBtn.onclick = () => {
        farms.sort((a, b) => (a.firstWeek - b.firstWeek) || (Number(a.id) - Number(b.id)));
        renderDesigner();
      };
    }
    const simulateBtn = E("#simulateBtn");
    if (simulateBtn) simulateBtn.onclick = simulate;
  }

  function init() {
    setupTabs();
    setupDesignerActions();
    farms = initialFarms();
    renderDesigner();
  }

  window.addEventListener("DOMContentLoaded", init);
})();