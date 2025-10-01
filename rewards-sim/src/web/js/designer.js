(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util;
  const ST = App.state;
  const S = ST.state;

  function setStatus(msg) {
    U.E("#status").textContent = msg || "";
  }

  function formatMoneyUSD(num) {
    const n = Number(num) || 0;
    return (n >= 1000 ? "$" + U.addCommas(Math.trunc(n)) : "$" + n.toFixed(2));
  }
  function formatPriceUSD2(num) {
    const n = Number(num) || 0;
    return "$" + n.toFixed(2);
  }

  function renderCompSelector() {
    const sel = U.E("#compSelect");
    if (!sel) return;
    sel.innerHTML = "";
    for (const c of S.competitions) {
      const opt = document.createElement("option");
      opt.value = c.key;
      opt.textContent = `${c.regionId} / ${c.assetId.toUpperCase()}`;
      if (c.key === S.selectedCompKey) opt.selected = true;
      sel.appendChild(opt);
    }
    sel.onchange = () => {
      S.selectedCompKey = sel.value;
      S.addMode = false;
      renderDesigner();
    };
  }

  function farmCardView(f) {
    const container = document.createElement("div");
    container.className = "card" + (f.edit ? " editing" : "");

    if (f.edit) {
      const header = document.createElement("div");
      header.className = "card-header";
      header.innerHTML = `<div class="card-title">Farm ${U.escapeHtml(f.id)} (edit)</div>`;
      container.appendChild(header);

      const form = document.createElement("div");
      form.className = "inline-form";
      form.innerHTML = `
        <label>Farm ID<input type="text" value="${U.escapeHtml(f.id)}" data-key="id"></label>
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
        U.Es("input", form).forEach(inp => {
          const key = inp.getAttribute("data-key");
          const val = inp.value;
          if (key === "id") {
            const newId = String(val || "").trim() || f.id;
            if (ST.globalFarmIdExists(newId, f)) {
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
        if (Number.isFinite(asNum)) S.nextId = Math.max(S.nextId, asNum + 1);
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
      header.innerHTML = `<div class="card-title">Farm ${U.escapeHtml(f.id)}</div>`;
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
        const c = ST.currentComp();
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
    if (!S.addMode) {
      add.textContent = "+ Add a farm";
      add.onclick = () => { S.addMode = true; renderDesigner(); };
      parent.appendChild(add);
      return;
    }

    const f = ST.defaultFarm();
    const title = document.createElement("div");
    title.className = "card-header";
    title.innerHTML = `<div class="card-title">Add a farm</div>`;
    add.appendChild(title);

    const form = document.createElement("div");
    form.className = "inline-form";
    form.innerHTML = `
      <label>Farm ID<input type="text" value="${U.escapeHtml(f.id)}" data-key="id"></label>
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
      U.Es("input", form).forEach(inp => {
        const key = inp.getAttribute("data-key");
        const val = inp.value;
        if (key === "id") {
          const nid = String(val || "").trim() || f.id;
          if (ST.globalFarmIdExists(nid, null)) {
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
      if (Number.isFinite(asNum)) S.nextId = Math.max(S.nextId, asNum + 1);
      obj.edit = false;
      const c = ST.currentComp();
      c.farms.push(obj);
      S.addMode = false;
      renderDesigner();
    };

    const cancel = document.createElement("button");
    cancel.className = "btn btn-secondary";
    cancel.textContent = "Cancel";
    cancel.onclick = () => { S.addMode = false; renderDesigner(); };

    actions.append(submit, cancel);
    add.appendChild(actions);

    parent.appendChild(add);
  }

  function addCompetitionPrompt() {
    const regionId = prompt("Enter region id for competition (e.g. simulation):", "simulation");
    if (!regionId) return;
    const assetId = prompt("Enter asset id for competition (e.g. glw):", "glw");
    if (!assetId) return;
    const key = U.keyOf(regionId, assetId);
    if (ST.findComp(key)) {
      setStatus("Competition already exists.");
      return;
    }
    S.competitions.push({ regionId, assetId, key, farms: [] });
    S.selectedCompKey = key;
    renderDesigner();
  }

  function renderDesigner() {
    renderCompSelector();

    const holder = U.E("#farmCards");
    holder.innerHTML = "";

    const c = ST.currentComp();
    if (!c) return;
    c.farms.forEach(f => holder.appendChild(farmCardView(f)));
    renderAddCard(holder);
  }

  function setupDesignerActions() {
    const sortBtn = U.E("#sortBtn");
    if (sortBtn) {
      sortBtn.onclick = () => {
        const c = ST.currentComp();
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
    const simulateBtn = U.E("#simulateBtn");
    if (simulateBtn) simulateBtn.onclick = App.api.simulate;

    const addCompBtn = U.E("#addCompBtn");
    if (addCompBtn) addCompBtn.onclick = addCompetitionPrompt;

    const compSelect = U.E("#compSelect");
    if (compSelect) compSelect.onchange = () => {
      S.selectedCompKey = compSelect.value;
      renderDesigner();
    };
  }

  App.designer = {
    renderDesigner,
    setupDesignerActions,
    setStatus
  };
})();