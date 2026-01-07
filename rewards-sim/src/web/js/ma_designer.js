(function () {
  "use strict";
  const App = (self.App = self.App || {});
  const U = App.util;
  const S = App.maState.state;

  function setStatus(msg) {
    U.E("#status").textContent = msg || "";
  }

  function validateAssetSum(f) {
    const total = f.totalDeposit || 0;
    const sum = f.assets.reduce((acc, a) => acc + (a.amountUSD || 0), 0);
    return Math.abs(total - sum) < 0.01;
  }

  function renderAssetRow(a, idx, parentList, onChange) {
    const row = document.createElement("div");
    row.style.display = "grid";
    row.style.gridTemplateColumns = "80px 1fr 1fr auto";
    row.style.gap = "6px";
    row.style.marginBottom = "6px";
    
    // Asset ID
    const sel = document.createElement("select");
    ["GLW", "USDG", "SGCTL"].forEach(opt => {
      const o = document.createElement("option");
      o.value = opt;
      o.textContent = opt;
      if (opt === a.assetId) o.selected = true;
      sel.appendChild(o);
    });
    sel.onchange = () => { a.assetId = sel.value; onChange(); };
    
    // Price
    const priceInp = document.createElement("input");
    priceInp.type = "number";
    priceInp.step = "0.000001";
    priceInp.min = "0.000001";
    priceInp.value = a.price;
    priceInp.placeholder = "Price ($)";
    priceInp.onchange = () => { a.price = parseFloat(priceInp.value) || 0; onChange(); };

    // Amount USD
    const amtInp = document.createElement("input");
    amtInp.type = "number";
    amtInp.step = "0.01";
    amtInp.min = "0";
    amtInp.value = a.amountUSD;
    amtInp.placeholder = "Value ($)";
    amtInp.onchange = () => { a.amountUSD = parseFloat(amtInp.value) || 0; onChange(); };

    // Delete
    const delBtn = document.createElement("button");
    delBtn.className = "btn btn-secondary";
    delBtn.style.padding = "4px 8px";
    delBtn.textContent = "×";
    delBtn.onclick = () => {
      parentList.splice(idx, 1);
      onChange();
    };

    row.append(sel, priceInp, amtInp, delBtn);
    return row;
  }

  function renderEditForm(f, container, onSave, onCancel) {
    // Clone farm to edit buffer
    const buf = JSON.parse(JSON.stringify(f));

    const form = document.createElement("div");
    form.className = "inline-form";
    form.style.gridTemplateColumns = "1fr 1fr";
    
    const fields = `
      <label>Farm ID<input type="text" data-key="id" value="${U.escapeHtml(buf.id)}"></label>
      <label>Region ID<input type="number" data-key="regionId" value="${buf.regionId}"></label>
      <label>First Week<input type="number" data-key="firstWeek" value="${buf.firstWeek}"></label>
      <label>Weeks Alive<input type="number" data-key="weeksAlive" value="${buf.weeksAlive}"></label>
      <label>Weekly Impact<input type="number" step="0.000001" data-key="weeklyIA" value="${buf.weeklyIA}"></label>
      <label>Total Deposit ($)<input type="number" step="0.01" data-key="totalDeposit" value="${buf.totalDeposit}"></label>
    `;
    form.innerHTML = fields;
    
    // Assets Section
    const assetsContainer = document.createElement("div");
    assetsContainer.style.gridColumn = "1 / -1";
    assetsContainer.style.marginTop = "12px";
    assetsContainer.style.padding = "10px";
    assetsContainer.style.background = "var(--grey-light)";
    assetsContainer.style.border = "1px solid var(--grey-med)";
    assetsContainer.style.borderRadius = "8px";

    const assetsHeader = document.createElement("div");
    assetsHeader.style.display = "flex";
    assetsHeader.style.justifyContent = "space-between";
    assetsHeader.style.marginBottom = "8px";
    assetsHeader.innerHTML = "<strong>Assets</strong> <span id='sum-check' style='font-size:12px'></span>";
    
    const assetsList = document.createElement("div");
    
    const refreshAssets = () => {
      assetsList.innerHTML = "";
      buf.assets.forEach((a, idx) => {
        assetsList.appendChild(renderAssetRow(a, idx, buf.assets, refreshAssets));
      });
      updateSumCheck();
    };

    const updateSumCheck = () => {
      // Sync top fields first
      const inputs = U.Es("input", form);
      inputs.forEach(inp => {
         if (inp.getAttribute("data-key") === "totalDeposit") {
             buf.totalDeposit = parseFloat(inp.value) || 0;
         }
      });
      
      const sum = buf.assets.reduce((acc, a) => acc + (a.amountUSD || 0), 0);
      const diff = buf.totalDeposit - sum;
      const el = assetsContainer.querySelector("#sum-check");
      if (Math.abs(diff) < 0.01) {
        el.textContent = "Sum matches total OK";
        el.style.color = "green";
      } else {
        el.textContent = `Sum: $${sum.toFixed(2)} (Diff: ${diff.toFixed(2)})`;
        el.style.color = "red";
      }
    };

    const addAssetBtn = document.createElement("button");
    addAssetBtn.className = "btn btn-ghost";
    addAssetBtn.style.fontSize = "12px";
    addAssetBtn.style.marginTop = "8px";
    addAssetBtn.textContent = "+ Add Asset";
    addAssetBtn.onclick = () => {
      buf.assets.push({ assetId: "GLW", price: 0, amountUSD: 0 });
      refreshAssets();
    };

    assetsContainer.append(assetsHeader, assetsList, addAssetBtn);
    form.appendChild(assetsContainer);
    
    // Attach listeners to top level inputs to update validation
    const inputs = U.Es("input", form);
    inputs.forEach(inp => {
       inp.oninput = updateSumCheck; 
    });

    refreshAssets();
    container.appendChild(form);

    const actions = document.createElement("div");
    actions.className = "card-actions";
    
    const saveBtn = document.createElement("button");
    saveBtn.className = "btn btn-primary";
    saveBtn.textContent = "Save";
    saveBtn.onclick = () => {
      const inputs = U.Es("input", form);
      const newObj = { ...buf }; // buf has assets updated
      inputs.forEach(inp => {
        const k = inp.getAttribute("data-key");
        if (k === "id") newObj.id = inp.value;
        else if (k === "regionId" || k === "firstWeek" || k === "weeksAlive") newObj[k] = parseInt(inp.value) || 0;
        else newObj[k] = parseFloat(inp.value) || 0;
      });
      
      if (!validateAssetSum(newObj)) {
        setStatus("Error: Sum of asset values must equal total deposit.");
        return;
      }
      
      onSave(newObj);
    };

    const cancelBtn = document.createElement("button");
    cancelBtn.className = "btn btn-secondary";
    cancelBtn.textContent = "Cancel";
    cancelBtn.onclick = onCancel;

    actions.append(saveBtn, cancelBtn);
    container.appendChild(actions);
  }

  function farmCardView(f) {
    const container = document.createElement("div");
    container.className = "card" + (f.edit ? " editing" : "");
    container.style.overflow = "hidden"; // contain floats

    if (f.edit) {
      const header = document.createElement("div");
      header.className = "card-header";
      header.innerHTML = `<div class="card-title">Edit Farm</div>`;
      container.appendChild(header);

      renderEditForm(f, container, (newObj) => {
        Object.assign(f, newObj);
        f.edit = false;
        renderDesigner();
      }, () => {
        f.edit = false;
        renderDesigner();
      });
    } else {
      const header = document.createElement("div");
      header.className = "card-header";
      header.innerHTML = `<div class="card-title">${U.escapeHtml(f.id)}</div><div class="badge">Region ${f.regionId}</div>`;
      container.appendChild(header);

      const kv = document.createElement("div");
      kv.className = "kv";
      kv.innerHTML = `
        <div>Week Range<br><strong>${f.firstWeek} - ${f.firstWeek + f.weeksAlive - 1}</strong></div>
        <div>Weekly Impact<br><strong>${f.weeklyIA}</strong></div>
        <div>Total Deposit<br><strong>${U.formatDollarsScaled(BigInt(Math.round(f.totalDeposit * 1000000)))}</strong></div>
        <div style="grid-column: 1 / -1">Assets<br>
          <div style="display:flex;flex-wrap:wrap;gap:4px;margin-top:4px;">
            ${f.assets.map(a => 
              `<span class="badge" style="background:var(--grey-med);font-weight:normal">${a.assetId}: $${a.amountUSD.toFixed(2)}</span>`
            ).join("")}
          </div>
        </div>
      `;
      container.appendChild(kv);

      const actions = document.createElement("div");
      actions.className = "card-actions";
      const editBtn = document.createElement("button");
      editBtn.className = "btn btn-ghost";
      editBtn.textContent = "Edit";
      editBtn.onclick = () => { f.edit = true; renderDesigner(); };
      
      const delBtn = document.createElement("button");
      delBtn.className = "btn btn-secondary";
      delBtn.textContent = "Delete";
      delBtn.onclick = () => {
        S.farms = S.farms.filter(x => x !== f);
        renderDesigner();
      };
      
      actions.append(editBtn, delBtn);
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

    const header = document.createElement("div");
    header.className = "card-header";
    header.innerHTML = `<div class="card-title">New Farm</div>`;
    add.appendChild(header);

    const f = App.maState.defaultFarm();
    renderEditForm(f, add, (newObj) => {
       S.farms.push(newObj);
       S.addMode = false;
       renderDesigner();
    }, () => {
       S.addMode = false;
       renderDesigner();
    });
    
    parent.appendChild(add);
  }

  function renderDesigner() {
    const holder = U.E("#farmCards");
    holder.innerHTML = "";
    S.farms.forEach(f => holder.appendChild(farmCardView(f)));
    renderAddCard(holder);
  }

  function setupDesignerActions() {
    const simBtn = U.E("#simulateBtn");
    if (simBtn) simBtn.onclick = App.maApi.simulate;
  }

  App.maDesigner = {
    renderDesigner,
    setupDesignerActions,
    setStatus
  };
})();