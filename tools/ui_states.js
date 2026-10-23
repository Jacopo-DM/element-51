// Put the page served by tools/ui_states.py into one named state, so it can be
// captured. Run it in the page's console or with the browser pane's javascript
// tool, then take a screenshot. Usage: `await uiState("menu")`.
//   default      nothing open
//   menu         the add-node menu, at the middle of the canvas
//   flyout       the menu with User defined nodes and its Math submenu open
//   search       the menu with a search that finds nodes
//   search-none  the menu with a search that finds nothing
//   card-menu    the card menu of the first node's header icon
//   run          the graph has run: ok, error and skipped nodes, and the banner
// Resets anything left open first.
async function uiState(name) {
    const canvas = document.getElementById("canvas");
    const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    document.getElementById("error").setAttribute("hidden", "");
    const openMenu = () =>
        canvas.dispatchEvent(
            new MouseEvent("contextmenu", { clientX: 520, clientY: 220, bubbles: true, cancelable: true }),
        );
    const search = (text) => {
        const input = document.getElementById("menu-search");
        input.value = text;
        input.dispatchEvent(new Event("input", { bubbles: true }));
    };
    if (name === "menu") openMenu();
    if (name === "flyout") {
        openMenu();
        for (const label of ["User defined nodes", "Math"]) {
            const category = document.querySelector(`#menu-tree button[data-category="${label}"]`);
            category.dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
            category.parentElement.classList.add("open");
        }
    }
    if (name === "search") {
        openMenu();
        search("s");
    }
    if (name === "search-none") {
        openMenu();
        search("zzz");
    }
    if (name === "card-menu") {
        canvas.dispatchEvent(
            new MouseEvent("mousedown", { clientX: 237, clientY: 76, button: 0, bubbles: true }),
        );
        canvas.dispatchEvent(new MouseEvent("mouseup", { clientX: 237, clientY: 76, bubbles: true }));
    }
    if (name === "run") document.getElementById("run").click();
    await wait(400);
    return name;
}
