// Count the axe-core violations of the page served by tools/ui_states.py in each
// state. It loads axe from a local server on port 8769 (see DESIGN.md, the
// ui-design-pass spec, for where `axe.min.js` comes from). Run it in the page's
// console or with the browser pane's javascript tool. Returns, for each state, a
// list of "rule (impact): nodes".
(async () => {
    if (!window.axe) {
        await new Promise((resolve, reject) => {
            const script = document.createElement("script");
            script.src = "http://127.0.0.1:8769/axe.min.js";
            script.onload = resolve;
            script.onerror = () => reject(new Error("axe.min.js did not load"));
            document.head.appendChild(script);
        });
    }
    const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
    const scan = async () => {
        const result = await axe.run(document, {
            runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa", "best-practice"] },
        });
        return result.violations.map((v) => `${v.id} (${v.impact}): ${v.nodes.length}`);
    };
    const canvas = document.getElementById("canvas");
    const found = { default: await scan() };
    document.getElementById("run").click();
    await wait(600);
    found.run = await scan();
    canvas.dispatchEvent(new MouseEvent("contextmenu", { clientX: 520, clientY: 220, bubbles: true, cancelable: true }));
    const search = document.getElementById("menu-search");
    search.value = "zzz";
    search.dispatchEvent(new Event("input", { bubbles: true }));
    await wait(300);
    found.searchNone = await scan();
    return found;
})();
