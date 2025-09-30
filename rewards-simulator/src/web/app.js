(function () {
  "use strict";

  const E = (sel, root = document) => root.querySelector(sel);
  const Es = (sel, root = document) => Array.from(root.querySelectorAll(sel));

  const SCALE_BI = 1000000000000000000n;

  let farms = [];
  let nextId = 1;
  let diagnostics = null;
  let addMode = false;

  let selectedWeek = null;
  let selectedFarmId = null;

  function initialFarms() {
    const items = [
      { id: String(nextId++), firstWeek: 1, weeksAlive: 5, weeklyCC: 0.08, protocolDeposit: 40000, assetPrice: 0.30, edit: false },
      { id: String(nextId++), firstWeek: 2, weeksAlive: 5, weeklyCC: 0.10, protocolDeposit: 80000, assetPrice: 0.40, edit: false },
      { id: String(nextId++), firstWeek: 2, weeksAlive: 5, weeklyCC: 0.12, protocolDeposit: 50000, assetPrice: 0.40, edit: false },
    ];
    return items;
  }

  function defaultFarm() {
    return {
      id: String(nextId++),
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

  function addCommas(intStr) {
    return String(intStr).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  }

  function formatScaledRule(x) {
    const bi = toBI(x);
    const neg = bi < 0n;
    const abs = neg ? -bi : bi;
    const thousandScaled = 1000n * SCALE_BI;
    const intPart = abs / SCALE_BI;
    const frac = abs % SCALE_BI;

    if (abs < thousandScaled) {
      const frac2 = frac / pow10BI(16); // 18 - 2
      const s = (neg ? "-" : "") + intPart.toString() + "." + frac2.toString().padStart(2, "0");
      return s;
    } else {
      const s = (neg ? "-" : "") + addCommas(intPart.toString());
      return s;
    }
  }

  const formatDollarsScaled = (x) => "$" + formatScaledRule(x);
  const formatTokensScaled = (x) => formatScaledRule(x) + " GLW";

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
    const solarFarms = farms.map(f => {
      const wcc = toScaledIntString(f.weeklyCC, 18);
      const pd = toScaledIntString(f.protocolDeposit, 18);
      const ap = toScaledIntString(Number(f.assetPrice).toFixed(2), 18);

      const pdBI = BigInt(pd);
      const apBI = BigInt(ap || "1");
      const arScaled = (pdBI * bigPow10(18)) / (apBI === 0n ? 1n : apBI);

      return {
        farmId: String(f.id),
        assetId: "glw",
        regionId: "simulation",
        weeklyCarbonCredits: wcc,
        protocolDepositValue: pd,
        assetsRequired: arScaled.toString(),
        rewardsAddress: randomEthAddress(),
        firstWeek: Number(f.firstWeek),
        weeksAlive: Math.max(2, Number(f.weeksAlive))
      };
    });

    return {
      cgpLeftovers: {},
      solarFarms
    };
  }

  function formatMoneyUSD(num) {
    const n = Number(num) || 0;
    return (n >= 1000
      ? "$" + addCommas(Math.trunc(n))
      : "$" + n.toFixed(2));
  }
  function formatPriceUSD2(num) {
    const n = Number(num) || 0;
    return "$" + n.toFixed(2);
  }

  function escapeHtml(s) {
    return String(s).replace(/[&<>'"]/g, c => ({'&':"&amp;",'<':"&lt;",'>':"&gt;","'":"&#39;",'"':"&quot;"}[c]));
  }

  function farmCardView(f) {
    const container = document.createElement("div");
    container.className = "card" + (f.edit ? " editing" : "");

    if (f.edit) {
      const header = document.createElement("div");
      header.className = "card-header";
      header.innerHTML = `<div class="card-title">Farm ${escapeHtml(f.id)} (edit)</div>`;
      container.appendChild(header);

      const form = document.createElement("div");
      form.className = "inline-form";
      form.innerHTML = `
        <label>Farm ID<input type="text" value="${escapeHtml(f.id)}" data-key="id"></label>
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
        const newObj = { ...f };
        Es("input", form).forEach(inp => {
          const key = inp.getAttribute("data-key");
          const val = inp.value;
          if (key === "id") {
            const newId = String(val || "").trim() || f.id;
            newObj.id = newId;
          } else if (key === "firstWeek" || key === "weeksAlive") {
            newObj[key] = Math.max((key === "weeksAlive" ? 2 : 1), parseInt(val, 10) || 0);
          } else if (key === "assetPrice") {
            const v = parseFloat(val) || 0;
            newObj.assetPrice = Math.max(0.01, Math.round(v * 100) / 100);
          } else {
            newObj[key] = parseFloat(val) || 0;
          }
        });
        if (farms.some(x => x !== f && String(x.id) === String(newObj.id))) {
          setStatus("Farm ID already exists.");
          return;
        }
        const asNum = Number(newObj.id);
        if (Number.isFinite(asNum)) nextId = Math.max(nextId, asNum + 1);
        Object.assign(f, newObj);
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
      header.innerHTML = `<div class="card-title">Farm ${escapeHtml(f.id)}</div>`;
      container.appendChild(header);

      const kv = document.createElement("div");
      kv.className = "kv";
      kv.innerHTML = `
        <div>First week<br><strong>${Number(f.firstWeek)}</strong></div>
        <div>Weeks alive<br><strong>${Number(f.weeksAlive)}</strong></div>
        <div>Weekly CC<br><strong>${Number(f.weeklyCC).toFixed(2)}</strong></div>
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
    const title = document.createElement("div");
    title.className = "card-header";
    title.innerHTML = `<div class="card-title">Add a farm</div>`;
    add.appendChild(title);

    const form = document.createElement("div");
    form.className = "inline-form";
    form.innerHTML = `
      <label>Farm ID<input type="text" value="${escapeHtml(f.id)}" data-key="id"></label>
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
          const nid = String(val || "").trim() || f.id;
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
      if (farms.some(x => String(x.id) === String(obj.id))) {
        setStatus("Farm ID already exists.");
        return;
      }
      const asNum = Number(obj.id);
      if (Number.isFinite(asNum)) nextId = Math.max(nextId, asNum + 1);
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
    selectedWeek = null;
    selectedFarmId = null;

    const body = buildApiInput();
    if (!body.solarFarms.length) {
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

  function computeDepositsRecovered(totalDepositsBI, farmCCBI, totalCCBI) {
    const td = toBI(totalDepositsBI);
    const fcc = toBI(farmCCBI);
    const tcc = toBI(totalCCBI) || 1n;
    return (td * fcc) / tcc;
  }

  function findPrevNetOver(comp, week, farmId) {
    const prevWeek = Number(week) - 1;
    const prevBucket = (comp.buckets || []).find(b => Number(b.weekNumber) === prevWeek);
    if (!prevBucket) return 0n;
    const st = (prevBucket.farmStates || []).find(s => String(s.farmId) === String(farmId));
    if (!st) return 0n;
    return toBI(st.netOverperformance);
  }

  function computePoolAndOwnGLW(comp, bucket, st, depRecBI) {
    const depositsContrib = toBI(st.depositsContributed);
    const curNetOver = toBI(st.netOverperformance);
    const prevNetOver = findPrevNetOver(comp, bucket.weekNumber, st.farmId);
    let baseOver = 0n;
    if (depRecBI > depositsContrib) {
      baseOver += depRecBI - depositsContrib;
    }
    baseOver += prevNetOver;
    baseOver -= curNetOver;
    if (baseOver < 0n) baseOver = 0n;

    const poolNetAssets = toBI(bucket.poolNetAssets);
    const poolNetDeposits = toBI(bucket.poolNetDeposits);
    let glwFromPool = 0n;
    if (poolNetDeposits > 0n) {
      glwFromPool = (baseOver * poolNetAssets) / poolNetDeposits;
      if (glwFromPool < 0n) glwFromPool = 0n;
    }
    const weekRewards = toBI(st.rewardsThisWeek);
    let glwFromOwn = weekRewards - glwFromPool;
    if (glwFromOwn < 0n) glwFromOwn = 0n;
    return { glwFromPool, glwFromOwn };
  }

  function updateWeekSelectionHighlight() {
    Es("#weekCards .card").forEach(c => {
      c.classList.toggle("selected", String(c.getAttribute("data-week")) === String(selectedWeek));
    });
  }
  function updateFarmSelectionHighlight() {
    Es("#farmSummaryCards .card").forEach(c => {
      c.classList.toggle("selected", String(c.getAttribute("data-fid")) === String(selectedFarmId));
    });
  }

  function renderPerWeek() {
    if (!diagnostics) return;
    const comps = diagnostics.competitions || [];
    const weeksMap = new Map();

    for (const comp of comps) {
      for (const b of comp.buckets) {
        const w = b.weekNumber;
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
        agg.total_deposits += toBI(b.totalDeposits);
        agg.total_carbon += toBI(b.totalCarbonCredits);
        agg.pool_assets += toBI(b.poolNetAssets);
        agg.pool_deposits += toBI(b.poolNetDeposits);
        const states = Array.isArray(b.farmStates) ? b.farmStates : [];
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
      card.setAttribute("data-week", String(w));
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Week ${w}</div>
          <div class="badge">${item.participants} farms</div>
        </div>
      `;
      card.style.cursor = "pointer";
      card.onclick = () => {
        selectedWeek = String(w);
        updateWeekSelectionHighlight();
        renderWeekDetails(w, item);
      };
      weekCards.appendChild(card);
    }
    if (sortedWeeks.length) {
      selectedWeek = String(sortedWeeks[0]);
      updateWeekSelectionHighlight();
      renderWeekDetails(sortedWeeks[0], weeksMap.get(sortedWeeks[0]));
    }
  }

  function renderWeekHeadline(weekNumber, agg) {
    const head = E("#weekHeadline");
    head.classList.add("centered");
    head.innerHTML = "";
    const wrap = document.createElement("div");
    wrap.className = "card highlight wide";
    wrap.innerHTML = `
      <div class="card-header">
        <div class="card-title">Week ${weekNumber} Overview</div>
      </div>
      <div class="kv">
        <div>Total deposits<br><strong>${formatDollarsScaled(agg.total_deposits)}</strong></div>
        <div>Total carbon<br><strong>${formatScaledRule(agg.total_carbon)}</strong></div>
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
      const finfo = (comp.farms || []).find(x => x.farmId === st.farmId);
      const kind = (weekNumber === finfo.firstWeek) ? "first" : (weekNumber === finfo.finalWeek ? "last" : "ongoing");

      const depRec = computeDepositsRecovered(bucket.totalDeposits, st.carbonCreditsContributed, bucket.totalCarbonCredits);
      const parts = computePoolAndOwnGLW(comp, bucket, st, depRec);

      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Farm ${escapeHtml(st.farmId)}</div>
          <span class="badge ${kind}">${kind}</span>
        </div>
        <div class="kv">
          <div>Deposits contributed<br><strong>${formatDollarsScaled(st.depositsContributed)}</strong></div>
          <div>Carbon contributed<br><strong>${formatScaledRule(st.carbonCreditsContributed)}</strong></div>

          <div>Deposits recovered<br><strong>${formatDollarsScaled(depRec)}</strong></div>
          <div>Rewards this week<br><strong>${formatTokensScaled(st.rewardsThisWeek)}</strong></div>

          <div>From own vault<br><strong>${formatTokensScaled(parts.glwFromOwn)}</strong></div>
          <div>From pool<br><strong>${formatTokensScaled(parts.glwFromPool)}</strong></div>

          <div>Accum. drawdown<br><strong>${formatDollarsScaled(st.accumulatedDrawdown)}</strong></div>
          <div>Net overperf.<br><strong>${formatDollarsScaled(st.netOverperformance)}</strong></div>
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
        for (const st of b.farmStates || []) {
          const fid = st.farmId;
          if (!farmMap.has(fid)) {
            const finfo = (comp.farms || []).find(x => x.farmId === fid) || {};
            farmMap.set(fid, { meta: { ...finfo }, entries: [] });
          }
          const rec = farmMap.get(fid);
          rec.entries.push({ comp, b, st, week: b.weekNumber });
        }
      }
    }

    const farmArr = Array.from(farmMap.entries()).map(([fid, v]) => ({ fid, ...v }))
      .sort((a,b)=>{
        const an = Number(a.fid), bn = Number(b.fid);
        if (Number.isFinite(an) && Number.isFinite(bn)) return an - bn;
        return String(a.fid).localeCompare(String(b.fid));
      });

    const holder = E("#farmSummaryCards");
    holder.innerHTML = "";
    for (const f of farmArr) {
      const card = document.createElement("div");
      card.className = "card compact";
      card.style.cursor = "pointer";
      card.setAttribute("data-fid", String(f.fid));
      const deposit = f.meta && f.meta.protocolDepositValue ? formatDollarsScaled(f.meta.protocolDepositValue) : "$0.00";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Farm ${escapeHtml(f.fid)}</div>
          <div class="badge">${deposit}</div>
        </div>
      `;
      card.onclick = () => {
        selectedFarmId = String(f.fid);
        updateFarmSelectionHighlight();
        renderFarmDetails(f);
      };
      holder.appendChild(card);
    }
    if (farmArr.length) {
      selectedFarmId = String(farmArr[0].fid);
      updateFarmSelectionHighlight();
      renderFarmDetails(farmArr[0]);
    }
  }

  function renderFarmHeadline(farmObj) {
    const h = E("#farmHeadline");
    h.classList.add("centered");
    h.innerHTML = "";
    const m = farmObj.meta || {};

    // Aggregate totals for the overview
    const entries = (farmObj.entries || []).slice().sort((a, b) => a.week - b.week);
    let totalRewards = 0n;
    let totalFromPool = 0n;
    for (const e of entries) {
      totalRewards += toBI(e.st.rewardsThisWeek);
      const depRecBI = computeDepositsRecovered(e.b.totalDeposits, e.st.carbonCreditsContributed, e.b.totalCarbonCredits);
      const parts = computePoolAndOwnGLW(e.comp, e.b, e.st, depRecBI);
      totalFromPool += parts.glwFromPool;
    }

    const card = document.createElement("div");
    card.className = "card highlight wide";
    card.innerHTML = `
      <div class="card-header">
        <div class="card-title">Farm ${escapeHtml(farmObj.fid)} Overview</div>
      </div>
      <div class="kv">
        <div>Total deposit<br><strong>${formatDollarsScaled(m.protocolDepositValue || 0)}</strong></div>
        <div>Assets required<br><strong>${formatTokensScaled(m.assetsRequired || 0)}</strong></div>
        <div>Total rewards<br><strong>${formatTokensScaled(totalRewards)}</strong></div>
        <div>Rewards from pool<br><strong>${formatTokensScaled(totalFromPool)}</strong></div>
      </div>
    `;
    h.appendChild(card);
  }

  function renderFarmDetails(farmObj) {
    renderFarmHeadline(farmObj);
    const d = E("#farmDetails");
    d.innerHTML = "";
    const entries = (farmObj.entries || []).sort((a,b) => a.week - b.week);

    for (const e of entries) {
      const b = e.b;
      const st = e.st;
      const depRecBI = computeDepositsRecovered(b.totalDeposits, st.carbonCreditsContributed, b.totalCarbonCredits);
      const parts = computePoolAndOwnGLW(e.comp, b, st, depRecBI);

      const kind = e.week === farmObj.meta.firstWeek ? "first" : (e.week === farmObj.meta.finalWeek ? "last" : "ongoing");

      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Week ${e.week}</div>
          <span class="badge ${kind}">${kind}</span>
        </div>
        <div class="kv">
          <div>Total deposits<br><strong>${formatDollarsScaled(b.totalDeposits)}</strong></div>
          <div>Total carbon<br><strong>${formatScaledRule(b.totalCarbonCredits)}</strong></div>

          <div>Farm deposits<br><strong>${formatDollarsScaled(st.depositsContributed)}</strong></div>
          <div>Farm carbon<br><strong>${formatScaledRule(st.carbonCreditsContributed)}</strong></div>

          <div>Deposits recovered<br><strong>${formatDollarsScaled(depRecBI)}</strong></div>
          <div>Rewards this week<br><strong>${formatTokensScaled(st.rewardsThisWeek)}</strong></div>

          <div>From own vault<br><strong>${formatTokensScaled(parts.glwFromOwn)}</strong></div>
          <div>From pool<br><strong>${formatTokensScaled(parts.glwFromPool)}</strong></div>

          <div>Accum. drawdown<br><strong>${formatDollarsScaled(st.accumulatedDrawdown)}</strong></div>
          <div>Net overperf.<br><strong>${formatDollarsScaled(st.netOverperformance)}</strong></div>

          <div>Pool net assets<br><strong>${formatTokensScaled(b.poolNetAssets)}</strong></div>
          <div>Pool net deposits<br><strong>${formatDollarsScaled(b.poolNetDeposits)}</strong></div>
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
        farms.sort((a, b) => {
          const fw = (a.firstWeek - b.firstWeek);
          if (fw) return fw;
          const an = Number(a.id), bn = Number(b.id);
          if (Number.isFinite(an) && Number.isFinite(bn)) return an - bn;
          return String(a.id).localeCompare(String(b.id));
        });
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

    const wc = E("#weekCards");
    if (wc) wc.classList.add("compact-grid");
    const fc = E("#farmSummaryCards");
    if (fc) fc.classList.add("compact-grid");
  }

  window.addEventListener("DOMContentLoaded", init);
})();