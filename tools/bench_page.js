// Time the page on the graph served by tools/bench_page.py (200 nodes).
// Run it in the page's console, or with the browser pane's javascript tool.
// The redraw batches run back to back without yielding, so this times the work
// the page does to record each frame, not the browser's painting of it; that
// keeps the numbers steady and the same in a hidden or a visible tab.
// Returns the median and the range of each measure, in milliseconds:
//   redraw: one wheel event, which zooms about the pointer and redraws the
//           canvas, averaged over a batch of 100 events;
//   load:   a fresh copy of the page in an iframe, from setting its address to
//           the page being shown (the class `ready` on the root element).
(async () => {
    const RUNS = 11;
    const EVENTS = 100;
    const summary = (values) => {
        const sorted = [...values].sort((a, b) => a - b);
        const round = (value) => Math.round(value * 100) / 100;
        return {
            median: round(sorted[Math.floor(sorted.length / 2)]),
            min: round(sorted[0]),
            max: round(sorted[sorted.length - 1]),
        };
    };
    const canvas = document.getElementById("canvas");
    const wheel = (deltaY) =>
        canvas.dispatchEvent(
            new WheelEvent("wheel", {
                deltaY,
                clientX: 400,
                clientY: 300,
                bubbles: true,
                cancelable: true,
            }),
        );
    const redraw = [];
    for (let run = 0; run < RUNS; run += 1) {
        const began = performance.now();
        for (let event = 0; event < EVENTS; event += 1) {
            wheel(event % 2 === 0 ? 40 : -40);
        }
        redraw.push((performance.now() - began) / EVENTS);
    }
    const load = [];
    for (let run = 0; run < RUNS; run += 1) {
        const frame = document.createElement("iframe");
        frame.style.cssText = "position: fixed; width: 800px; height: 600px; left: -2000px";
        document.body.appendChild(frame);
        const began = performance.now();
        const elapsed = await new Promise((resolve) => {
            frame.addEventListener("load", () => {
                const root = frame.contentDocument.documentElement;
                const done = () => resolve(performance.now() - began);
                if (root.classList.contains("ready")) {
                    done();
                    return;
                }
                const watcher = new MutationObserver(() => {
                    if (root.classList.contains("ready")) {
                        watcher.disconnect();
                        done();
                    }
                });
                watcher.observe(root, { attributes: true });
            });
            frame.src = location.href;
        });
        load.push(elapsed);
        frame.remove();
    }
    return { redraw: summary(redraw), load: summary(load) };
})();
