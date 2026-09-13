import { byId } from '../data';
import { NS, svg } from './svg';
import { genealogyMode, labelWidth } from '../state';
export const LINE_HEIGHT = 13;
const cache = new Map<string, {lines: string[]; truncated: boolean; retained: number; width:number; height:number; characterHeight:number; idWidth:number}>();
export function nodeLabel(id: string) {
    const widthLimit=genealogyMode ? 156 : labelWidth;
    const key=id+' / '+widthLimit;
    if (cache.has(key)) return cache.get(key);
    const probe = document.createElementNS(NS, 'text') as SVGTextElement;
    probe.setAttribute('class','nlabel');
    svg.appendChild(probe);
    const chars = Array.from(String(byId[id].title || ''));
    const lines: string[] = [];
    let i=0;
    for(let row=0;row<3 && i<chars.length;row++) {
        let line='';
        while(i<chars.length) {
            probe.textContent=line+chars[i]+(row===2 && i+1<chars.length?'…':'');
            if(line && probe.getComputedTextLength()>widthLimit) break;
            line+=chars[i++];
        }
        lines.push(line);
    }
    const retained=i, truncated=i<chars.length;
    if(truncated) lines[lines.length-1]+='…';
    let width=0;
    for(const line of [id,...lines]) {probe.textContent=line;width=Math.max(width,probe.getComputedTextLength());}
    probe.textContent=id;
    const idWidth=probe.getComputedTextLength(),characterHeight=probe.getBBox().height;
    probe.remove();
    const result={lines,truncated,retained,width,height:LINE_HEIGHT*(1+lines.length),characterHeight,idWidth};
    cache.set(key,result);
    return result;
}
export function labelBounds(id: string) {
    const label=nodeLabel(id);
    return {left:Math.min(-16,22-label.width/2),right:Math.max(16,22+label.width/2,18+label.idWidth),top:-16,bottom:26+Math.max(0,label.lines.length-1)*LINE_HEIGHT+4};
}

const clusterWidths=new Map<string,number>();
export function clusterWidth(group: string, count: number) {
    const text=group.replace('\u0000',' / ')+' · '+count;
    if(!clusterWidths.has(text)) {
        const probe=document.createElementNS(NS,'text') as SVGTextElement;
        probe.setAttribute('class','nlabel');probe.textContent=text;svg.appendChild(probe);
        clusterWidths.set(text,Math.max(120,probe.getComputedTextLength()+24));probe.remove();
    }
    return clusterWidths.get(text);
}

export function packingPlan(ids: string[], viewport: {width:number;height:number}) {
    const width=Math.max(...ids.map(id=>{const b=labelBounds(id);return b.right-b.left;}))+6;
    const height=Math.max(...ids.map(id=>{const b=labelBounds(id);return b.bottom-b.top;}))+6;
    let columns=1,fit=0;
    for(let count=1;count<=ids.length;count++) {
        const candidate=Math.min((viewport.width-12)/(count*width),(viewport.height-12)/(Math.ceil(ids.length/count)*height));
        if(candidate>fit) {columns=count;fit=candidate;}
    }
    return {width,height,columns,fit};
}
