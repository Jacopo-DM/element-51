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
    const pressWith = async (key, modifiers) => {
        document.activeElement.dispatchEvent(
            new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...modifiers }),
        );
        await wait(150);
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
        check("Enter on a free input with no link started opens its value box", active().id === `field-${target.node}-${target.id.split("-input-")[1]}` && /^Type a value for /.test(status.textContent), active().id + " / " + status.textContent);
        await pressWith("Escape", {});
        check("Escape in the box goes back to the port", active().id === target.id, active().id);
        await goTo(source);
        await press("Enter");
        check("Enter on an output starts a link", /^Link from n\d+ \w+ started/.test(status.textContent), status.textContent);
        await press("Escape");
        check("Escape cancels it", status.textContent === "Link cancelled", status.textContent);
        await goTo(target);
        await press("Enter");
        check("with the link cancelled, Enter on an input opens its box again", /^Type a value for /.test(status.textContent), status.textContent);
        await pressWith("Escape", {});

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
            check("a refused link ends the pending link", /^Type a value for /.test(status.textContent), status.textContent);
            await pressWith("Escape", {});
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

    // Task 30: move, pan and zoom by keyboard.
    nodes()[0].focus();
    await wait(60);
    const movedId = active().dataset.node;
    await pressWith("ArrowRight", { shiftKey: true });
    check("Shift+Arrow moves the node one step and says so", status.textContent === "Moved " + movedId, status.textContent);
    await pressWith("ArrowDown", { shiftKey: true, altKey: true });
    check("Alt+Shift+Arrow moves it four steps and says so", status.textContent === "Moved " + movedId, status.textContent);
    check("the node keeps the keyboard focus after a move", tree.contains(active()) && active().dataset.node === movedId, active().id);
    for (let i = 0; i < 2; i++) {
        document.activeElement.dispatchEvent(new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true, cancelable: true }));
        await wait(300);
    }
    check("two Ctrl+Z undo the two moves", status.textContent === "Undid the last change", status.textContent);
    document.activeElement.dispatchEvent(new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true, cancelable: true }));
    await wait(300);
    check("each move was one undo step (nothing is left to undo)", status.textContent === "Nothing to undo", status.textContent);

    await pressWith("ArrowLeft", { altKey: true });
    check("Alt+Arrow pans and says which way", status.textContent === "Panned left", status.textContent);
    await pressWith("ArrowUp", { altKey: true });
    check("Alt+Up pans up", status.textContent === "Panned up", status.textContent);
    await pressWith("+", { shiftKey: true });
    check("+ zooms in", status.textContent === "Zoom 125 percent", status.textContent);
    await pressWith("+", { shiftKey: true });
    check("a second + zooms again", status.textContent === "Zoom 156 percent", status.textContent);
    await pressWith("-");
    check("- zooms out", status.textContent === "Zoom 125 percent", status.textContent);
    await pressWith("0");
    check("0 goes back to 100 percent", status.textContent === "Zoom 100 percent", status.textContent);
    await pressWith("ArrowRight", { altKey: true });
    await pressWith("ArrowDown", { altKey: true });
    check("panning back leaves the tree focused", tree.contains(active()), active().id);

    // Task 31: the add-node menu and the card menu by keyboard.
    const menu = document.getElementById("menu");
    const cardMenu = document.getElementById("card-menu");
    nodes()[0].focus();
    await wait(60);
    const opener = active().id;
    await pressWith("n", {});
    check("N opens the add-node menu with the cursor in its search box", !menu.hidden && active().id === "menu-search", active().id);
    check("opening it is announced", /^Add node menu/.test(status.textContent), status.textContent);
    await pressWith("ArrowDown", {});
    check("Down from the search box goes to the first item", active().closest("#menu-tree") !== null && active().tagName === "BUTTON", active().textContent);
    const category = menu.querySelector("#menu-tree button[data-category]");
    check("the menu has a category to open", category !== null, category && category.dataset.category);
    if (category) {
        category.focus();
        await wait(60);
        await pressWith("ArrowRight", {});
        const panel = category.parentElement.querySelector(":scope > .panel");
        check("Right opens the flyout and moves into it", category.getAttribute("aria-expanded") === "true" && panel.contains(active()), active().textContent);
        check("the flyout is shown", getComputedStyle(panel).display !== "none", getComputedStyle(panel).display);
        await pressWith("ArrowDown", {});
        await pressWith("ArrowLeft", {});
        check("Left closes it and goes back to the category", active() === category && category.getAttribute("aria-expanded") === "false", active().textContent);
        await pressWith("ArrowRight", {});
        await pressWith("Escape", {});
        check("Escape closes only the flyout first", !menu.hidden && active() === category, active().textContent);
    }
    await pressWith("Escape", {});
    check("Escape closes the menu and returns to where the focus was", menu.hidden && active().id === opener, active().id + " vs " + opener);

    await pressWith("n", {});
    await pressWith("ArrowDown", {});
    await pressWith("ArrowRight", {});
    for (let i = 0; i < 12 && active().dataset.nodeType === undefined; i++) await pressWith("ArrowDown", {});
    const picked = active().dataset.nodeType;
    const countBefore = nodes().length;
    if (picked !== undefined) active().click();
    await wait(400);
    check("picking an item adds a node and says so", nodes().length === countBefore + 1 && /^Added node n\d+, /.test(status.textContent), nodes().length + " / " + status.textContent);
    check("the menu closed and the focus came back", menu.hidden && tree.contains(active()), active().id);
    document.activeElement.dispatchEvent(new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true, cancelable: true }));
    await wait(300);
    check("Ctrl+Z removes the added node", nodes().length === countBefore, nodes().length);

    nodes()[0].focus();
    await wait(60);
    const cardOpener = active().id;
    await pressWith("F10", { shiftKey: true });
    check("Shift+F10 opens the node's card menu", !cardMenu.hidden && cardMenu.contains(active()), active().textContent);
    check("it says which node", /^Actions for n\d+$/.test(status.textContent), status.textContent);
    await pressWith("ArrowDown", {});
    check("Down stays inside the card menu", cardMenu.contains(active()), active().textContent);
    await pressWith("Escape", {});
    check("Escape closes it and returns to the node", cardMenu.hidden && active().id === cardOpener, active().id);
    await pressWith("ContextMenu", {});
    check("the Menu key opens it too", !cardMenu.hidden, cardMenu.hidden);
    await pressWith("Escape", {});

    // Task 32: value boxes by keyboard.
    const boxPort = ports.find((port) => port.side === "input" && /not linked|linked from/.test(port.label) && document.getElementById("field-" + port.node + "-" + port.id.split("-input-")[1]));
    const linkedPort = ports.find((port) => port.side === "input" && /linked from/.test(port.label) && !document.getElementById("field-" + port.node + "-" + port.id.split("-input-")[1]));
    check("an input with a value box and a linked input were found", Boolean(boxPort && linkedPort), (boxPort && boxPort.id) + " / " + (linkedPort && linkedPort.id));
    if (boxPort && linkedPort) {
        const fieldId = "field-" + boxPort.node + "-" + boxPort.id.split("-input-")[1];
        await goTo(boxPort);
        await press("Enter");
        const box = document.getElementById(fieldId);
        check("Enter on an unlinked input puts the cursor in its value box", active() === box, active().id);
        check("the box is named for its port and node", /of node n\d+$/.test(box.getAttribute("aria-label")), box.getAttribute("aria-label"));
        const was = box.value;
        box.value = "9";
        await pressWith("Escape", {});
        check("Escape puts the old value back and returns to the port", box.value === was && active().id === boxPort.id, box.value + " / " + active().id);
        await press("Enter");
        const box2 = document.getElementById(fieldId);
        box2.value = "7";
        // A script-set value does not make the browser fire `change` when the
        // box blurs (typing does), so fire it by hand.
        box2.dispatchEvent(new Event("change", { bubbles: true }));
        await pressWith("Enter", {});
        await wait(300);
        check("Enter in the box sends the value and says so", /^Set .+ of n\d+ to 7$/.test(status.textContent), status.textContent);
        check("and returns to the input port", active().id === boxPort.id, active().id);
        document.activeElement.dispatchEvent(new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true, cancelable: true }));
        await wait(300);
        check("Ctrl+Z takes the value away", status.textContent === "Undid the last change", status.textContent);
        await goTo(linkedPort);
        await press("Enter");
        check("Enter on a linked input says it has no box", /^That input is linked/.test(status.textContent) && active().id === linkedPort.id, status.textContent);
    }

    // Task 33: the end of a run is announced.
    status.textContent = "";
    document.getElementById("run").click();
    await wait(800);
    check("a run says how many nodes were ok, failed and skipped", /^Run finished: \d+ ok(, \d+ errors?)?(, \d+ skipped)?/.test(status.textContent), status.textContent);
    check("it names the first error when there is one", !/error/.test(status.textContent) || /First error: n\d+: /.test(status.textContent), status.textContent);
    document.getElementById("error").setAttribute("hidden", "");

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
