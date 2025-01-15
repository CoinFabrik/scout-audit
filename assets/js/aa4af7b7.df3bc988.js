"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[1544],{9966:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>d,contentTitle:()=>a,default:()=>u,frontMatter:()=>o,metadata:()=>i,toc:()=>c});const i=JSON.parse('{"id":"detectors/ink/iterators-over-indexing","title":"Iterators over indexing","description":"What it does","source":"@site/docs/detectors/ink/19-iterators-over-indexing.md","sourceDirName":"detectors/ink","slug":"/detectors/ink/iterators-over-indexing","permalink":"/scout-audit/docs/detectors/ink/iterators-over-indexing","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/ink/19-iterators-over-indexing.md","tags":[],"version":"current","sidebarPosition":19,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Unprotected self destruct","permalink":"/scout-audit/docs/detectors/ink/unprotected-self-destruct"},"next":{"title":"Ink! version","permalink":"/scout-audit/docs/detectors/ink/ink-version"}}');var s=t(5105),r=t(6755);const o={},a="Iterators over indexing",d={},c=[{value:"What it does",id:"what-it-does",level:3},{value:"Why is this bad?",id:"why-is-this-bad",level:3},{value:"Example",id:"example",level:3},{value:"Implementation",id:"implementation",level:3}];function l(e){const n={a:"a",code:"code",h1:"h1",h3:"h3",header:"header",p:"p",pre:"pre",...(0,r.R)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(n.header,{children:(0,s.jsx)(n.h1,{id:"iterators-over-indexing",children:"Iterators over indexing"})}),"\n",(0,s.jsx)(n.h3,{id:"what-it-does",children:"What it does"}),"\n",(0,s.jsxs)(n.p,{children:["It warns if a ",(0,s.jsx)(n.code,{children:"for"})," loop uses indexing instead of an iterator. If the indexing goes to ",(0,s.jsx)(n.code,{children:".len()"})," it will not warn."]}),"\n",(0,s.jsx)(n.h3,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,s.jsx)(n.p,{children:"Accessing a vector by index is slower than using an iterator. Also, if the index is out of bounds, it will panic."}),"\n",(0,s.jsx)(n.h3,{id:"example",children:"Example"}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-rust",children:"    #[ink(message)]\n    pub fn bad_indexing(&self){\n        for i in 0..3 {\n            foo(self.value[i]);\n        }\n    }\n"})}),"\n",(0,s.jsx)(n.p,{children:"Use instead:"}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-rust",children:"   #[ink(message)]\n   pub fn iterator(&self) {\n       for item in self.value.iter() {\n            foo(self.value[i]);\n       }\n   }\n\n// or if its not iterable (with `in`, `iter` or `to_iter()`)\n\n   #[ink(message)]\n   pub fn index_to_len(&self){\n       for i in 0..self.value.len() {\n            foo(self.value[i]);\n       }\n"})}),"\n",(0,s.jsx)(n.h3,{id:"implementation",children:"Implementation"}),"\n",(0,s.jsxs)(n.p,{children:["The detector's implementation can be found at ",(0,s.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout/tree/main/detectors/iterators-over-indexing",children:"this link"}),"."]})]})}function u(e={}){const{wrapper:n}={...(0,r.R)(),...e.components};return n?(0,s.jsx)(n,{...e,children:(0,s.jsx)(l,{...e})}):l(e)}},6755:(e,n,t)=>{t.d(n,{R:()=>o,x:()=>a});var i=t(8101);const s={},r=i.createContext(s);function o(e){const n=i.useContext(r);return i.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function a(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:o(e.components),i.createElement(r.Provider,{value:n},e.children)}}}]);