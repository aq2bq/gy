interface Controls {
    'q': HTMLInputElement;
    'scopeSel': HTMLSelectElement;
    'stateSel': HTMLSelectElement;
    'qstatus': HTMLSelectElement;
    'criterion': HTMLSelectElement;
    'hopFrom': HTMLInputElement;
    'hopRadius': HTMLSelectElement;
    'applyHop': HTMLButtonElement;
    'clearFilter': HTMLButtonElement;
    'genealogy': HTMLButtonElement;
    'zin': HTMLButtonElement;
    'zout': HTMLButtonElement;
    'zfit': HTMLButtonElement;
    'acChart': SVGSVGElement;
    'qChart': SVGSVGElement;
    'focusBack': HTMLButtonElement;
    'focusAll': HTMLButtonElement;
    'focusDetail': HTMLButtonElement;
    'svg': SVGSVGElement;
    'closeDetail': HTMLButtonElement;
}
export function element<K extends keyof Controls>(id: K): Controls[K];
export function element(id: string): HTMLElement;
export function element(id: string): HTMLElement | SVGSVGElement {
    return document.getElementById(id);
}
