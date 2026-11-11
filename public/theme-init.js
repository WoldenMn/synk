(function () {
  function read(key) {
    try {
      var raw = localStorage.getItem(key);
      return raw === null ? null : raw.replace(/^"|"$/g, "");
    } catch (e) {
      return null;
    }
  }
  var theme = read("synk-theme") || "dark";
  var accent = read("synk-color-theme");
  var root = document.documentElement;
  if (theme === "dark") root.classList.add("dark");
  if (accent && /^(indigo|emerald|rose|amber|slate)$/.test(accent)) {
    root.classList.add("theme-" + accent);
  }
})();
