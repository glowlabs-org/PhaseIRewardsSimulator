(function () {
  "use strict";
  const App = (self.App = self.App || {});

  const SCALE_TOKENS_18 = 1000000000000000000n;
  const SCALE_DOLLARS_6 = 1000000n;

  const E = (sel, root = document) => root.querySelector(sel);
  const Es = (sel, root = document) => Array.from(root.querySelectorAll(sel));

  function keyOf(regionId, assetId) {
    return String(regionId) + "::" + String(assetId);
  }

  function addCommas(intStr) {
    return String(intStr).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  }

  function formatScaledGeneric(x, scaleBI) {
    const bi = toBI(x);
    const neg = bi < 0n;
    const abs = neg ? -bi : bi;
    const scaleSafe = scaleBI > 0n ? scaleBI : 1n;
    const thousandScaled = 1000n * scaleSafe;
    const intPart = abs / scaleSafe;
    const fracRaw = abs % scaleSafe;

    if (abs < thousandScaled) {
      let frac2 = 0n;
      if (scaleSafe > 1n) {
        frac2 = (fracRaw * 100n) / scaleSafe;
      }
      return (neg ? "-" : "") + intPart.toString() + "." + frac2.toString().padStart(2, "0");
    } else {
      return (neg ? "-" : "") + addCommas(intPart.toString());
    }
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

  function pow10BI(n) {
    return BigInt("1" + "0".repeat(Number(n)));
  }
  function bigPow10(n) {
    return BigInt("1" + "0".repeat(n));
  }

  function randomEthAddress() {
    const hex = [...crypto.getRandomValues(new Uint8Array(20))]
      .map(b => b.toString(16).padStart(2, "0"))
      .join("");
    return "0x" + hex;
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
          const v = typeof limbs[i] === "number" ? BigInt(Math.trunc(limbs[i])) : BigInt(String(limbs[i]).replace(/[^\d]/g, "") || "0");
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
            const v = typeof limbs[i] === "number" ? BigInt(Math.trunc(limbs[i])) : BigInt(String(limbs[i]).replace(/[^\d]/g, "") || "0");
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

  const formatDollarsScaled = (x) => "$" + formatScaledGeneric(x, SCALE_DOLLARS_6);
  const formatImpactScaled = (x) => formatScaledGeneric(x, SCALE_TOKENS_18);
  const formatTokensScaled = (x, assetId) => {
    const ticker = String(assetId || "glw").toUpperCase();
    const scale = String(assetId).toLowerCase() === "usdg" ? SCALE_DOLLARS_6 : SCALE_TOKENS_18;
    return formatScaledGeneric(x, scale) + " " + ticker;
  };
  const formatUnscaled = (x) => formatScaledGeneric(x, 1n);
  const formatGlwUnscaled = (x) => formatUnscaled(x) + " GLW";

  function escapeHtml(s) {
    return String(s).replace(/[&<>'"]/g, c => ({'&':"&amp;",'<':"&lt;",'>':"&gt;","'":"&#39;",'"':"&quot;"}[c]));
  }

  App.util = {
    E, Es,
    SCALE_TOKENS_18, SCALE_DOLLARS_6,
    keyOf, addCommas,
    formatScaledGeneric, formatDollarsScaled, formatImpactScaled, formatTokensScaled, formatUnscaled, formatGlwUnscaled,
    toScaledIntString, pow10BI, bigPow10, randomEthAddress,
    toBI, escapeHtml
  };
})();