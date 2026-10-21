// Walk the editor's hidden tree by keyboard and check what it says. Run it in
// the page of tools/ui_states.py (or any example) with the browser pane's
// javascript tool: paste the function, then `await uiKeys()`.
// The page must have keyboard focus (click empty canvas first), or the browser
// sends no focus events and rows that depend on them fail. Reload the page and
// restart the server before a run: the rows delete and restore a node and a link.
// Each row is [step, passed, what was seen]. The page's window must have the
// focus, or the browser sends no focus events and the canvas ring never moves:
// click the empty canvas first (take a screenshot before a click by coordinate).
// The ring is not in the DOM, so look at it in a screenshot after `uiKeys()`,
// which stops on a port of the first node.
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
    check("ports are not built until entered", tree.querySelectorAll("ul").length === 0, tree.querySelectorAll("ul").length);
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
    check(
        "leaving a node closes its ports",
        first.getAttribute("aria-expanded") === "false" && first.querySelector("ul").hidden,
        first.getAttribute("aria-expanded"),
    );
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

    // Task 28: select, delete and undo, with the status line.
    const status = document.getElementById("status");
    check("the status line is a live region", status && status.getAttribute("role") === "status", status && status.id);
    nodes()[0].focus();
    await wait(60);
    await press("Enter");
    check("Enter selects the focused node", /^Selected n\d+$/.test(status.textContent), status.textContent);
    await press(" ");
    check("Space selects it too", /^Selected n\d+$/.test(status.textContent), status.textContent);

    // Find an input port with a link: walk the nodes, entering their ports.
    let linked = null;
    for (const li of nodes()) {
        li.focus();
        await wait(30);
        await press("ArrowRight");
        for (let i = 0; i < 6 && !linked; i++) {
            if (active().dataset.side === "input" && /linked from/.test(active().getAttribute("aria-label"))) linked = active().id;
            else await press("ArrowDown");
        }
        if (linked) break;
    }
    check("a linked input port exists to try", linked !== null, linked);
    const linkLabel = linked && document.getElementById(linked).getAttribute("aria-label");
    await press("Delete");
    check("Delete on a linked input removes its link", /^Deleted the link from /.test(status.textContent), status.textContent);
    const gone = document.getElementById(linked);
    if (gone) gone.focus();
    await wait(60);
    check("the port now says it is not linked", gone && /not linked/.test(document.getElementById(linked).getAttribute("aria-label")), document.getElementById(linked) && document.getElementById(linked).getAttribute("aria-label"));
    document.activeElement.dispatchEvent(new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true, cancelable: true }));
    await wait(300);
    check("Ctrl+Z puts the link back and says so", status.textContent === "Undid the last change", status.textContent);
    const restored = document.getElementById(linked);
    check("the port is linked again", restored && restored.getAttribute("aria-label") === linkLabel, restored && restored.getAttribute("aria-label"));

    // Delete on an output port explains itself.
    const firstNode = nodes()[0];
    firstNode.focus();
    await wait(30);
    await press("ArrowRight");
    for (let i = 0; i < 6 && active().dataset.side !== "output"; i++) await press("ArrowDown");
    if (active().dataset.side === "output") await press("Delete");
    check("Delete on an output port says why nothing happened", /^Delete works on a node/.test(status.textContent), status.textContent);

    // Delete a node, then undo it.
    const count = nodes().length;
    nodes().at(-1).focus();
    await wait(30);
    const lastId = active().dataset.node;
    await press("Delete");
    await wait(300);
    check("Delete removes the node and says so", status.textContent === "Deleted node " + lastId && nodes().length === count - 1, status.textContent + " / " + nodes().length);
    check("the focus moved to a neighbour, not lost", tree.contains(active()) || active().id === "run", active().id);
    document.activeElement.dispatchEvent(new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true, cancelable: true }));
    await wait(400);
    check("Ctrl+Z brings the node back", nodes().length === count && status.textContent === "Undid the last change", nodes().length + " / " + status.textContent);

    // Task 29: link by keyboard. Take an input that is linked, free it with
    // Delete, then link it again by keyboard (and try a link the rules refuse).
    const ports = [];
    for (const li of nodes()) {
        li.focus();
        await wait(30);
        await press("ArrowRight");
        let last = "";
        for (let i = 0; i < 12 && active().id !== last; i++) {
            last = active().id;
            if (active().dataset.port !== undefined)
                ports.push({ id: active().id, node: active().dataset.node, side: active().dataset.side, label: active().getAttribute("aria-label") });
            await press("ArrowDown");
        }
        await press("ArrowLeft");
    }
    const kind = (port) => /, (float|int|str|bool),/.exec(port.label)[1];
    const fits = (from, to) => from === to || (from === "int" && to === "float");
    const goTo = async (port) => {
        document.getElementById("tree-" + port.node).focus();
        await wait(30);
        await press("ArrowRight");
        for (let i = 0; i < 12 && active().id !== port.id; i++) await press("ArrowDown");
        return active().id === port.id;
    };
    const target = ports.find((port) => port.side === "input" && /linked from n\d+ \w+/.test(port.label));
    const from = target && /linked from (n\d+) (\w+)/.exec(target.label);
    const source = from && ports.find((port) => port.id === `tree-${from[1]}-output-${from[2]}`);
    const clashing = target && ports.find((port) => port.side === "output" && port.node !== target.node && !fits(kind(port), kind(target)));
    check("a linked input and its source were found", Boolean(target && source), target && target.id);
    check("an output whose type does not fit that input was found", Boolean(clashing), clashing && clashing.id);

    if (target && source) {
        await goTo(target);
        await press("Delete");
        await wait(300);
        await goTo(target);
        await press("Enter");
        check("Enter on an input with no link started says so", /^No link started/.test(status.textContent), status.textContent);
        await goTo(source);
        await press("Enter");
        check("Enter on an output starts a link", /^Link from n\d+ \w+ started/.test(status.textContent), status.textContent);
        await press("Escape");
        check("Escape cancels it", status.textContent === "Link cancelled", status.textContent);
        await goTo(target);
        await press("Enter");
        check("after Escape an input finds no link started", /^No link started/.test(status.textContent), status.textContent);

        if (clashing) {
            document.getElementById("error").setAttribute("hidden", "");
            await goTo(clashing);
            await press("Enter");
            await goTo(target);
            await press("Enter");
            await wait(300);
            check("a link the rules refuse is refused with the reason", /^Refused: /.test(status.textContent), status.textContent);
            check("the refusal also shows in the banner", !document.getElementById("error").hidden, document.getElementById("error").hidden);
            await press("Enter");
            check("a refused link ends the pending link", /^No link started/.test(status.textContent), status.textContent);
            document.getElementById("error").setAttribute("hidden", "");
        }

        await goTo(source);
        await press("Enter");
        await goTo(target);
        await press("Enter");
        await wait(300);
        check("Enter on a fitting input links, and says so", /^Linked n\d+ \w+ to n\d+ \w+$/.test(status.textContent), status.textContent);
        const linkedNow = document.getElementById(target.id);
        check("the input says it is linked again", linkedNow && linkedNow.getAttribute("aria-label") === target.label, linkedNow && linkedNow.getAttribute("aria-label"));
        for (let i = 0; i < 2; i++) {
            document.activeElement.dispatchEvent(new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true, cancelable: true }));
            await wait(300);
        }
        check("two Ctrl+Z put the graph back as it was", status.textContent === "Undid the last change", status.textContent);
    }

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
