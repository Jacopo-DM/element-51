// Walk the editor's hidden tree by keyboard and check what it says. Run it in
// the page of tools/ui_states.py (or any example) with the browser pane's
// javascript tool, then read the table it returns: `await uiKeys()`.
// Each row is [step, passed, what was seen]. The focus ring on the canvas is not
// in the DOM, so look at it in a screenshot after `uiKeys()` stops on a port.
async function uiKeys() {
    const rows = [];
    const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
    const check = (step, passed, seen) => rows.push([step, Boolean(passed), String(seen)]);
    const tree = document.getElementById("graph-tree");
    const press = async (key) => {
        document.activeElement.dispatchEvent(
            new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }),
        );
        await wait(60);
    };
    const active = () => document.activeElement;
    const nodes = () => [...tree.children];

    check("the tree exists and shows", tree && !tree.hidden && tree.getAttribute("role") === "tree", tree && tree.hidden);
    check("one item per node", nodes().length > 0 && nodes().every((li) => li.dataset.node), nodes().length);
    check(
        "exactly one tab stop",
        tree.querySelectorAll('[tabindex="0"]').length === 1,
        tree.querySelectorAll('[tabindex="0"]').length,
    );

    nodes()[0].focus();
    await wait(60);
    check("focus lands on the first node", active() === nodes()[0], active().id);
    check(
        "a node is named by id, type and port counts",
        /^n\d+, .+\. \d+ inputs?, \d+ outputs?\.$/.test(active().getAttribute("aria-label")),
        active().getAttribute("aria-label"),
    );

    await press("ArrowRight");
    const onPort = active().dataset.port !== undefined;
    check("Right enters the ports", onPort, active().getAttribute("aria-label"));
    check(
        "the node reports itself expanded",
        active().closest('[aria-expanded="true"]') !== null,
        active().parentElement.parentElement.getAttribute("aria-expanded"),
    );
    check(
        "a port says its type and its links",
        /^(Input|Output) .+, (float|int|str|bool), (not linked|linked (from|to) .+)$/.test(active().getAttribute("aria-label")),
        active().getAttribute("aria-label"),
    );
    const portId = active().id;
    await press("ArrowDown");
    check("Down moves to another port or stays on the last", active().dataset.port !== undefined, active().id);
    await press("ArrowLeft");
    check("Left goes back to the node", active().dataset.port === undefined && active().dataset.node, active().id);

    const first = nodes()[0];
    await press("PageDown");
    check("PageDown goes to the next node in reading order", nodes().length < 2 || active() === first.nextElementSibling, active().id);
    await press("End");
    check("End goes to the last node", active() === nodes().at(-1), active().id);
    await press("Home");
    check("Home goes to the first node", active() === first, active().id);

    const before = active().id;
    for (const key of ["ArrowDown", "ArrowRight", "ArrowLeft", "ArrowUp"]) await press(key);
    check("arrows always land on an item of the tree", tree.contains(active()), active().id + " (from " + before + ")");

    check(
        "collapsed nodes keep their ports out of the tab order",
        [...tree.querySelectorAll('ul[hidden] [tabindex="0"]')].length === 0,
        "",
    );
    check("still one tab stop", tree.querySelectorAll('[tabindex="0"]').length === 1, tree.querySelectorAll('[tabindex="0"]').length);

    // Leave focus on a port of the first node, for a screenshot of the ring.
    first.focus();
    await wait(60);
    await press("ArrowRight");
    document.getElementById("error").setAttribute("hidden", "");
    return {
        passed: rows.filter((row) => row[1]).length,
        failed: rows.filter((row) => !row[1]).length,
        stoppedOn: portId + " then " + active().id,
        rows,
    };
}
