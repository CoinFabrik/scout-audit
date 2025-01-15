"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[3181],{9147:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>a,contentTitle:()=>c,default:()=>h,frontMatter:()=>s,metadata:()=>o,toc:()=>d});const o=JSON.parse('{"id":"detectors/ink/integer-overflow-or-underflow","title":"Integer overflow or underflow","description":"What it does","source":"@site/docs/detectors/ink/1-integer-overflow-or-underflow.md","sourceDirName":"detectors/ink","slug":"/detectors/ink/integer-overflow-or-underflow","permalink":"/scout-audit/docs/detectors/ink/integer-overflow-or-underflow","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/ink/1-integer-overflow-or-underflow.md","tags":[],"version":"current","sidebarPosition":1,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Scout Audit Detectors","permalink":"/scout-audit/docs/detectors/detectors-intro"},"next":{"title":"Set contract storage","permalink":"/scout-audit/docs/detectors/ink/set-contract-storage"}}');var r=n(5105),i=n(6755);const s={},c="Integer overflow or underflow",a={},d=[{value:"What it does",id:"what-it-does",level:3},{value:"Why is this bad?",id:"why-is-this-bad",level:3},{value:"Known problems",id:"known-problems",level:3},{value:"Example",id:"example",level:3},{value:"Implementation",id:"implementation",level:3}];function l(e){const t={a:"a",code:"code",h1:"h1",h3:"h3",header:"header",p:"p",pre:"pre",...(0,i.R)(),...e.components};return(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(t.header,{children:(0,r.jsx)(t.h1,{id:"integer-overflow-or-underflow",children:"Integer overflow or underflow"})}),"\n",(0,r.jsx)(t.h3,{id:"what-it-does",children:"What it does"}),"\n",(0,r.jsxs)(t.p,{children:["Checks for integer arithmetic operations which could overflow or panic.\nSpecifically, checks for any operators (",(0,r.jsx)(t.code,{children:"+"}),", ",(0,r.jsx)(t.code,{children:"-"}),", ",(0,r.jsx)(t.code,{children:"*"}),", ",(0,r.jsx)(t.code,{children:"<<"}),", etc) which are capable\nof overflowing according to the ",(0,r.jsx)(t.a,{href:"https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow",children:"Rust\nReference"}),",\nor which can panic (",(0,r.jsx)(t.code,{children:"/"}),", ",(0,r.jsx)(t.code,{children:"%"}),"). No bounds analysis or sophisticated reasoning is\nattempted."]}),"\n",(0,r.jsx)(t.h3,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,r.jsx)(t.p,{children:"Integer overflow will trigger a panic in debug builds or will wrap in\nrelease mode. Division by zero will cause a panic in either mode. In some applications one\nwants explicitly checked, wrapping or saturating arithmetic."}),"\n",(0,r.jsx)(t.h3,{id:"known-problems",children:"Known problems"}),"\n",(0,r.jsx)(t.h3,{id:"example",children:"Example"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-rust",children:"let a = 0;\nlet b = a + 1;\n"})}),"\n",(0,r.jsx)(t.p,{children:"Use instead:"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-rust",children:"let a = 0;\nlet b = a.checked_add(1).ok_or(Error::OverflowDetected)?;\n"})}),"\n",(0,r.jsx)(t.h3,{id:"implementation",children:"Implementation"}),"\n",(0,r.jsxs)(t.p,{children:["The detector's implementation can be found at ",(0,r.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout/tree/main/detectors/integer-overflow-or-underflow",children:"this link"}),"."]})]})}function h(e={}){const{wrapper:t}={...(0,i.R)(),...e.components};return t?(0,r.jsx)(t,{...e,children:(0,r.jsx)(l,{...e})}):l(e)}},6755:(e,t,n)=>{n.d(t,{R:()=>s,x:()=>c});var o=n(8101);const r={},i=o.createContext(r);function s(e){const t=o.useContext(i);return o.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function c(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:s(e.components),o.createElement(i.Provider,{value:t},e.children)}}}]);