const $ = document.querySelector.bind(document);
const $$ = document.querySelectorAll.bind(document);

window.onload = () => {
    for (const tr of $$(".table-row")) {
        const tableRow: HTMLTableRowElement = tr;
        tableRow.onclick = () => {
            const aElement: HTMLAnchorElement = tableRow.getElementsByTagName("a")[0];
            window.location.href = aElement.getAttribute("href");
        };
    }
}