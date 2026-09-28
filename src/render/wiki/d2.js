(() => {
  const button = (text, title, onclick) => {
    const b = document.createElement("button");
    Object.assign(b, { textContent: text, title, onclick });
    return b;
  };

  const zoom = (svg) => {
    const width = Number(svg.getAttribute("viewBox").split(" ")[2]);
    const frame = document.createElement("div");
    const pane = document.createElement("div");
    const bar = document.createElement("div");
    const view = svg.cloneNode(true);
    let scale = 1;
    let drag = null;
    let moved = false;

    const fit = () => view.setAttribute("width", width * scale);
    const by = (f) => {
      const cx = (pane.scrollLeft + pane.clientWidth / 2) / pane.scrollWidth;
      const cy = (pane.scrollTop + pane.clientHeight / 2) / pane.scrollHeight;
      scale = Math.min(8, Math.max(0.1, scale * f));
      fit();
      pane.scrollLeft = cx * pane.scrollWidth - pane.clientWidth / 2;
      pane.scrollTop = cy * pane.scrollHeight - pane.clientHeight / 2;
    };
    const key = (e) => {
      if (e.key === "Escape") close();
      if (e.key === "+" || e.key === "=") by(1.25);
      if (e.key === "-") by(0.8);
    };
    const close = () => {
      frame.remove();
      document.body.style.overflow = "";
      document.removeEventListener("keydown", key);
    };

    frame.className = "d2-zoom";
    pane.className = "d2-zoom-pane";
    bar.className = "d2-zoom-bar";
    view.removeAttribute("height");
    fit();
    bar.append(button("+", "Zoom in", () => by(1.25)), button("−", "Zoom out", () => by(0.8)), button("×", "Close", close));
    pane.append(view);
    frame.append(pane, bar);
    pane.addEventListener("wheel", (e) => {
      if (!e.ctrlKey && !e.metaKey) return;
      e.preventDefault();
      by(e.deltaY < 0 ? 1.1 : 0.9);
    }, { passive: false });
    pane.addEventListener("pointerdown", (e) => {
      drag = [e.clientX, e.clientY, pane.scrollLeft, pane.scrollTop];
      moved = false;
    });
    pane.addEventListener("pointermove", (e) => {
      if (!drag) return;
      const [x, y, left, top] = drag;
      moved ||= Math.abs(e.clientX - x) + Math.abs(e.clientY - y) > 4;
      pane.scrollLeft = left - (e.clientX - x);
      pane.scrollTop = top - (e.clientY - y);
    });
    pane.addEventListener("pointerup", () => (drag = null));
    pane.addEventListener("click", (e) => moved && (e.preventDefault(), e.stopPropagation()), true);
    document.addEventListener("keydown", key);
    document.body.style.overflow = "hidden";
    document.body.append(frame);
  };

  document.querySelectorAll(".d2").forEach((d) => {
    const svg = d.querySelector(":scope > svg");
    if (svg) d.prepend(button("⤢", "Expand", () => zoom(svg)));
  });
})();
