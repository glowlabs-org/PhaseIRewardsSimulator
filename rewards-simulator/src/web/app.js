(function () {
  "use strict";

  const E = (sel, root = document) => root.querySelector(sel);
  const Es = (sel, root = document) => Array.from(root.querySelectorAll(sel));

  const SCALE_TOKENS_18 = 1000000000000000000n; // 1e18 for non-usdg assets and impact assets
  const SCALE_DOLLARS_6 = 1000000n; // 1e6 for USD-denominated values and USDG token

  function keyOf(regionId, assetId) {
    return String(regionId) + "::" + String(assetId);
  }

  let competitions = []; // [{key, regionId, assetId, farms: [Farm]}]
  let selectedCompKey = null;

  let diagnostics = null;
  let selectedVizCompKey = null;

  let addMode = false;
  let nextId = 1;

  let selectedWeek = null;
  let selectedFarmId = null;

  function initialFarms() {
    const items = [
      { id: String(nextId++), firstWeek: 1, weeksAlive: 5, weeklyIA: 0.08, protocolDeposit: 40000, assetPrice: 0.30, edit: false },
      { id: String(nextId++), firstWeek: 2, weeksAlive: 5, weeklyIA: 0.10, protocolDeposit: 80000, assetPrice: 0.40, edit: false },
      { id: String(nextId++), firstWeek: 2, weeksAlive: 5, weeklyIA: 0.12, protocolDeposit: 50000, assetPrice: 0.40, edit: false },
    ];
    return items;
  }

  function ensureInitialData() {
    if (competitions.length) return;
    const comp = {
      regionId: "simulation",
      assetId: "glw",
      key: keyOf("simulation", "glw"),
      farms: initialFarms(),
    };
    competitions.push(comp);
    selectedCompKey = comp.key;
  }

  function defaultFarm() {
    return {
      id: String(nextId++),
      firstWeek: 1,
      weeksAlive: 5,
      weeklyIA: 0.08,
      protocolDeposit: 40000,
      assetPrice: 0.30,
      edit: false,
    };
  }

  function findComp(key) {
    return competitions.find(c => c.key === key) || null;
  }
  function currentComp() {
    return findComp(selectedCompKey);
  }
  function currentFarms() {
    const c = currentComp();
    return c ? c.farms : [];
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

  function formatScaledGeneric(x, scaleBI) {
    const bi = toBI(x);
    const neg = bi < 0n;
    const abs = neg ? -bi : bi;
    const thousandScaled = 1000n * scaleBI;
    const intPart = abs / scaleBI;
    const frac = abs % scaleBI;

    if (abs < thousandScaled) {
      const scaleDigits = String(scaleBI).length - 1;
      const fracShown = 2;
      const fracDiv = pow10BI(BigInt(scaleDigits - fracShown));
      const frac2 = frac / fracDiv;
      const s = (neg ? "-" : "") + intPart.toString() + "." + frac2.toString().padStart(fracShown, "0");
      return s;
    } else {
      const s = (neg ? "-" : "") + addCommas(intPart.toString());
      return s;
    }
  }

  const formatDollarsScaled = (x) => "$" + formatScaledGeneric(x, SCALE_DOLLARS_6);
  const formatImpactScaled = (x) => formatScaledGeneric(x, SCALE_TOKENS_18);
  const formatTokensScaled = (x, assetId) => {
    const ticker = String(assetId || "glw").toUpperCase();
    const scale = String(assetId).toLowerCase() === "usdg" ? SCALE_DOLLARS_6 : SCALE_TOKENS_18;
    return formatScaledGeneric(x, scale) + " " + ticker;
  };

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

  function globalFarmIdExists(id, exceptObj) {
    for (const c of competitions) {
      for (const f of c.farms) {
        if (f === exceptObj) continue;
        if (String(f.id) === String(id)) return true;
      }
    }
    return false;
  }

  function buildApiInput() {
    const solarFarms = [];
    for (const comp of competitions) {
      for (const f of comp.farms) {
        const wia = toScaledIntString(f.weeklyIA, 18);

        // Dollars are scaled 1e6
        const pd = toScaledIntString(f.protocolDeposit, 6);
        const ap = toScaledIntString(Number(f.assetPrice).toFixed(2), 6);

        const pdBI = BigInt(pd);
        const apBI = BigInt(ap || "1");

        // assetsRequired scale: 1e18 normally, but 1e6 for USDG asset
        const tokenScale = (String(comp.assetId).toLowerCase() === "usdg")
          ? bigPow10(6)
          : bigPow10(18);
        const arScaled = (pdBI * tokenScale) / (apBI === 0n ? 1n : apBI);

        solarFarms.push({
          farmId: String(f.id),
          assetId: comp.assetId,
          regionId: comp.regionId,
          netWeeklyImpactAssets: wia,
          protocolDepositValue: pd,
          assetsRequired: arScaled.toString(),
          rewardsAddress: randomEthAddress(),
          firstWeek: Number(f.firstWeek),
          weeksAlive: Math.max(2, Number(f.weeksAlive))
        });
      }
    }

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
        <label>Weekly impact assets<input type="number" step="0.000001" min="0.000000000000000001" value="${f.weeklyIA}" data-key="weeklyIA"></label>
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
            if (globalFarmIdExists(newId, f)) {
              setStatus("Farm ID already exists.");
              return;
            }
            newObj.id = newId;
          } else if (key === "firstWeek" || key === "weeksAlive") {
            newObj[key] = Math.max((key === "weeksAlive" ? 2 : 1), parseInt(val, 10) || 0);
          } else if (key === "assetPrice") {
            const v = parseFloat(val) || 0;
            newObj.assetPrice = Math.max(0.01, Math.round(v * 100) / 100);
          } else if (key === "weeklyIA") {
            newObj.weeklyIA = parseFloat(val) || 0;
          } else {
            newObj[key] = parseFloat(val) || 0;
          }
        });
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
        <div>Weekly impact assets<br><strong>${Number(f.weeklyIA).toFixed(2)}</strong></div>
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
        const c = currentComp();
        c.farms = c.farms.filter(x => x !== f);
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
      <label>Weekly impact assets<input type="number" step="0.000001" min="0.000000000000000001" value="${f.weeklyIA}" data-key="weeklyIA"></label>
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
          if (globalFarmIdExists(nid, null)) {
            setStatus("Farm ID already exists.");
            return;
          }
          obj.id = nid;
        } else if (key === "firstWeek" || key === "weeksAlive") {
          obj[key] = Math.max((key === "weeksAlive" ? 2 : 1), parseInt(val, 10) || 0);
        } else if (key === "assetPrice") {
          const v = parseFloat(val) || 0;
          obj.assetPrice = Math.max(0.01, Math.round(v * 100) / 100);
        } else if (key === "weeklyIA") {
          obj.weeklyIA = parseFloat(val) || 0;
        } else {
          obj[key] = parseFloat(val) || 0;
        }
      });
      const asNum = Number(obj.id);
      if (Number.isFinite(asNum)) nextId = Math.max(nextId, asNum + 1);
      obj.edit = false;
      const c = currentComp();
      c.farms.push(obj);
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

  function renderCompSelector() {
    const sel = E("#compSelect");
    if (!sel) return;
    sel.innerHTML = "";
    for (const c of competitions) {
      const opt = document.createElement("option");
      opt.value = c.key;
      opt.textContent = `${c.regionId} / ${c.assetId.toUpperCase()}`;
      if (c.key === selectedCompKey) opt.selected = true;
      sel.appendChild(opt);
    }
    sel.onchange = () => {
      selectedCompKey = sel.value;
      addMode = false;
      renderDesigner();
    };
  }

  function addCompetitionPrompt() {
    const regionId = prompt("Enter region id for competition (e.g. simulation):", "simulation");
    if (!regionId) return;
    const assetId = prompt("Enter asset id for competition (e.g. glw):", "glw");
    if (!assetId) return;
    const key = keyOf(regionId, assetId);
    if (findComp(key)) {
      setStatus("Competition already exists.");
      return;
    }
    competitions.push({ regionId, assetId, key, farms: [] });
    selectedCompKey = key;
    renderDesigner();
  }

  function renderDesigner() {
    renderCompSelector();

    const holder = E("#farmCards");
    holder.innerHTML = "";

    const c = currentComp();
    if (!c) return;
    c.farms.forEach(f => holder.appendChild(farmCardView(f)));
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
      const preload = !!(E("#toggleV1") && E("#toggleV1").checked);
      const url = "/api/rewards-simulator-detailed" + (preload ? "?preloadGlowV1=true" : "");
      const res = await fetch(url, {
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
        setupVizCompSelector();
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

  function computeDepositsRecovered(totalDepositsBI, farmIABI, totalIABI) {
    const td = toBI(totalDepositsBI);
    const fia = toBI(farmIABI);
    const tia = toBI(totalIABI) || 1n;
    return (td * fia) / tia;
  }

  function findPrevNetOver(comp, week, farmId) {
    const prevWeek = Number(week) - 1;
    const prevBucket = (comp.buckets || []).find(b => Number(b.weekNumber) === prevWeek);
    if (!prevBucket) return 0n;
    const st = (prevBucket.farmStates || []).find(s => String(s.farmId) === String(farmId));
    if (!st) return 0n;
    return toBI(st.netOverperformance);
  }

  function computePoolAndOwnTokens(comp, bucket, st, depRecBI) {
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
    let tokensFromPool = 0n;
    if (poolNetDeposits > 0n) {
      tokensFromPool = (baseOver * poolNetAssets) / poolNetDeposits;
    }
    const weekRewards = toBI(st.rewardsThisWeek);
    let tokensFromOwn = weekRewards - tokensFromPool;
    if (tokensFromOwn < 0n) tokensFromOwn = 0n;
    return { tokensFromPool, tokensFromOwn };
  }

  function setupVizCompSelector() {
    const sel = E("#vizCompSelect");
    if (!sel || !diagnostics) return;
    const comps = Array.isArray(diagnostics.competitions) ? diagnostics.competitions : [];
    sel.innerHTML = "";
    for (const c of comps) {
      const k = keyOf(c.regionId, c.assetId);
      const opt = document.createElement("option");
      opt.value = k;
      opt.textContent = `${c.regionId} / ${String(c.assetId).toUpperCase()}`;
      sel.appendChild(opt);
    }
    if (!selectedVizCompKey || !comps.find(c => keyOf(c.regionId, c.assetId) === selectedVizCompKey)) {
      selectedVizCompKey = comps.length ? keyOf(comps[0].regionId, comps[0].assetId) : null;
    }
    if (selectedVizCompKey) sel.value = selectedVizCompKey;
    sel.onchange = () => {
      selectedVizCompKey = sel.value;
      selectedWeek = null;
      selectedFarmId = null;
      renderPerWeek();
      renderPerFarm();
    };
  }

  function getDiagnosticsComp() {
    if (!diagnostics) return null;
    const comps = Array.isArray(diagnostics.competitions) ? diagnostics.competitions : [];
    if (!comps.length) return null;
    if (!selectedVizCompKey) return comps[0];
    return comps.find(c => keyOf(c.regionId, c.assetId) === selectedVizCompKey) || comps[0];
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
    const comp = getDiagnosticsComp();
    const weeksMap = new Map();

    if (comp) {
      for (const b of comp.buckets || []) {
        const w = b.weekNumber;
        if (!weeksMap.has(w)) {
          weeksMap.set(w, {
            total_deposits: 0n,
            total_impact: 0n,
            pool_assets: 0n,
            pool_deposits: 0n,
            participants: 0,
            items: []
          });
        }
        const agg = weeksMap.get(w);
        agg.total_deposits += toBI(b.totalDeposits);
        agg.total_impact += toBI(b.totalImpactAssets);
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
        renderWeekDetails(w, item, comp);
      };
      weekCards.appendChild(card);
    }
    if (sortedWeeks.length) {
      selectedWeek = String(sortedWeeks[0]);
      updateWeekSelectionHighlight();
      renderWeekDetails(sortedWeeks[0], weeksMap.get(sortedWeeks[0]), comp);
    }
  }

  function renderWeekHeadline(weekNumber, agg, comp) {
    const head = E("#weekHeadline");
    head.classList.add("centered");
    head.innerHTML = "";
    const wrap = document.createElement("div");
    wrap.className = "card highlight wide";
    const compAsset = comp && comp.assetId ? comp.assetId : "glw";
    wrap.innerHTML = `
      <div class="card-header">
        <div class="card-title">Week ${weekNumber} Overview</div>
      </div>
      <div class="kv">
        <div>Total deposits<br><strong>${formatDollarsScaled(agg.total_deposits)}</strong></div>
        <div>Total impact assets<br><strong>${formatImpactScaled(agg.total_impact)}</strong></div>
        <div>Pool net assets<br><strong>${formatTokensScaled(agg.pool_assets, compAsset)}</strong></div>
        <div>Pool net deposits<br><strong>${formatDollarsScaled(agg.pool_deposits)}</strong></div>
      </div>
    `;
    head.appendChild(wrap);
  }

  function renderWeekDetails(weekNumber, agg, comp) {
    renderWeekHeadline(weekNumber, agg, comp);
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
      const { bucket, st } = it;
      const finfo = (comp.farms || []).find(x => x.farmId === st.farmId);
      const assetId = finfo ? finfo.assetId : "glw";
      const kind = (weekNumber === (finfo && finfo.firstWeek) ? "first" : (weekNumber === (finfo && finfo.finalWeek) ? "last" : "ongoing"));

      const depRec = computeDepositsRecovered(bucket.totalDeposits, st.impactAssetsContributed, bucket.totalImpactAssets);
      const parts = computePoolAndOwnTokens(comp, bucket, st, depRec);

      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML = `
        <div class="card-header">
          <div class="card-title">Farm ${escapeHtml(st.farmId)}</div>
          <span class="badge ${kind}">${kind}</span>
        </div>
        <div class="kv">
          <div>Deposits contributed<br><strong>${formatDollarsScaled(st.depositsContributed)}</strong></div>
          <div>Impact assets contributed<br><strong>${formatImpactScaled(st.impactAssetsContributed)}</strong></div>

          <div>Deposits recovered<br><strong>${formatDollarsScaled(depRec)}</strong></div>
          <div>Rewards this week<br><strong>${formatTokensScaled(st.rewardsThisWeek, assetId)}</strong></div>

          <div>From own vault<br><strong>${formatTokensScaled(parts.tokensFromOwn, assetId)}</strong></div>
          <div>From pool<br><strong>${formatTokensScaled(parts.tokensFromPool, assetId)}</strong></div>

          <div>Accum. drawdown<br><strong>${formatDollarsScaled(st.accumulatedDrawdown)}</strong></div>
          <div>Net overperf.<br><strong>${formatDollarsScaled(st.netOverperformance)}</strong></div>
        </div>
      `;
      details.appendChild(card);
    }
  }

  function renderPerFarm() {
    if (!diagnostics) return;
    const comp = getDiagnosticsComp();
    const farmMap = new Map();

    if (comp) {
      for (const b of comp.buckets || []) {
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

    const entries = (farmObj.entries || []).slice().sort((a, b) => a.week - b.week);
    let totalRewards = 0n;
    let totalFromPool = 0n;
    for (const e of entries) {
      totalRewards += toBI(e.st.rewardsThisWeek);
      const depRecBI = computeDepositsRecovered(e.b.totalDeposits, e.st.impactAssetsContributed, e.b.totalImpactAssets);
      const parts = computePoolAndOwnTokens(e.comp, e.b, e.st, depRecBI);
      totalFromPool += parts.tokensFromPool;
    }

    const card = document.createElement("div");
    card.className = "card highlight wide";
    card.innerHTML = `
      <div class="card-header">
        <div class="card-title">Farm ${escapeHtml(farmObj.fid)} Overview</div>
      </div>
      <div class="kv">
        <div>Total deposit<br><strong>${formatDollarsScaled(m.protocolDepositValue || 0)}</strong></div>
        <div>Assets required<br><strong>${formatTokensScaled(m.assetsRequired || 0, m.assetId || "glw")}</strong></div>
        <div>Total rewards<br><strong>${formatTokensScaled(totalRewards, m.assetId || "glw")}</strong></div>
        <div>Rewards from pool<br><strong>${formatTokensScaled(totalFromPool, m.assetId || "glw")}</strong></div>
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
      const depRecBI = computeDepositsRecovered(b.totalDeposits, st.impactAssetsContributed, b.totalImpactAssets);
      const parts = computePoolAndOwnTokens(e.comp, b, st, depRecBI);

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
          <div>Total impact assets<br><strong>${formatImpactScaled(b.totalImpactAssets)}</strong></div>

          <div>Farm deposits<br><strong>${formatDollarsScaled(st.depositsContributed)}</strong></div>
          <div>Farm impact assets<br><strong>${formatImpactScaled(st.impactAssetsContributed)}</strong></div>

          <div>Deposits recovered<br><strong>${formatDollarsScaled(depRecBI)}</strong></div>
          <div>Rewards this week<br><strong>${formatTokensScaled(st.rewardsThisWeek, farmObj.meta.assetId || "glw")}</strong></div>

          <div>From own vault<br><strong>${formatTokensScaled(parts.tokensFromOwn, farmObj.meta.assetId || "glw")}</strong></div>
          <div>From pool<br><strong>${formatTokensScaled(parts.tokensFromPool, farmObj.meta.assetId || "glw")}</strong></div>

          <div>Accum. drawdown<br><strong>${formatDollarsScaled(st.accumulatedDrawdown)}</strong></div>
          <div>Net overperf.<br><strong>${formatDollarsScaled(st.netOverperformance)}</strong></div>

          <div>Pool net assets<br><strong>${formatTokensScaled(b.poolNetAssets, farmObj.meta.assetId || "glw")}</strong></div>
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
        const c = currentComp();
        if (!c) return;
        c.farms.sort((a, b) => {
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

    const addCompBtn = E("#addCompBtn");
    if (addCompBtn) addCompBtn.onclick = addCompetitionPrompt;

    const compSelect = E("#compSelect");
    if (compSelect) compSelect.onchange = () => {
      selectedCompKey = compSelect.value;
      renderDesigner();
    };
  }

  function init() {
    ensureInitialData();
    setupTabs();
    setupDesignerActions();
    renderDesigner();

    const wc = E("#weekCards");
    if (wc) wc.classList.add("compact-grid");
    const fc = E("#farmSummaryCards");
    if (fc) fc.classList.add("compact-grid");
  }

  window.addEventListener("DOMContentLoaded", init);
})();