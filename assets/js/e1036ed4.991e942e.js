"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[3205],{247:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>l,contentTitle:()=>i,default:()=>u,frontMatter:()=>a,metadata:()=>o,toc:()=>c});const o=JSON.parse('{"id":"detectors/ink/avoid-core-mem-forget","title":"Avoid core::mem::forget usage","description":"What it does","source":"@site/docs/detectors/ink/16-avoid-core-mem-forget.md","sourceDirName":"detectors/ink","slug":"/detectors/ink/avoid-core-mem-forget","permalink":"/docs/detectors/ink/avoid-core-mem-forget","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/ink/16-avoid-core-mem-forget.md","tags":[],"version":"current","sidebarPosition":16,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Assert violation","permalink":"/docs/detectors/ink/assert-violation"},"next":{"title":"Avoid fromat! macro usage","permalink":"/docs/detectors/ink/avoid-format-string"}}');var s=n(5105),r=n(6755);const a={},i="Avoid core::mem::forget usage",l={},c=[{value:"What it does",id:"what-it-does",level:3},{value:"Why is this bad?",id:"why-is-this-bad",level:3},{value:"Example",id:"example",level:3},{value:"Implementation",id:"implementation",level:3}];function d(e){const t={a:"a",code:"code",h1:"h1",h3:"h3",header:"header",p:"p",pre:"pre",...(0,r.R)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(t.header,{children:(0,s.jsx)(t.h1,{id:"avoid-corememforget-usage",children:"Avoid core::mem::forget usage"})}),"\n",(0,s.jsx)(t.h3,{id:"what-it-does",children:"What it does"}),"\n",(0,s.jsxs)(t.p,{children:["Checks for ",(0,s.jsx)(t.code,{children:"core::mem::forget"})," usage."]}),"\n",(0,s.jsx)(t.h3,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,s.jsx)(t.p,{children:"This is a bad practice because it can lead to memory leaks, resource leaks and logic errors."}),"\n",(0,s.jsx)(t.h3,{id:"example",children:"Example"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:"   #[ink(message)]\n   pub fn forget_value(&mut self) {\n       let forgotten_value = self.value;\n       self.value = false;\n       core::mem::forget(forgotten_value);\n   }\n"})}),"\n",(0,s.jsx)(t.p,{children:"Use instead:"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:"   #[ink(message)]\n   pub fn forget_value(&mut self) {\n       let forgotten_value = self.value;\n       self.value = false;\n       let _ = forgotten_value;\n   }\n\n// or if droppable\n\n    #[ink(message)]\n    pub fn drop_value(&mut self) {\n        let forgotten_value = self.value;\n        self.value = false;\n        forget_value.drop();\n    }\n"})}),"\n",(0,s.jsx)(t.h3,{id:"implementation",children:"Implementation"}),"\n",(0,s.jsxs)(t.p,{children:["The detector's implementation can be found at ",(0,s.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout/tree/main/detectors/avoid-core-mem-forget",children:"this link"}),"."]})]})}function u(e={}){const{wrapper:t}={...(0,r.R)(),...e.components};return t?(0,s.jsx)(t,{...e,children:(0,s.jsx)(d,{...e})}):d(e)}},6755:(e,t,n)=>{n.d(t,{R:()=>a,x:()=>i});var o=n(8101);const s={},r=o.createContext(s);function a(e){const t=o.useContext(r);return o.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function i(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:a(e.components),o.createElement(r.Provider,{value:t},e.children)}}}]);