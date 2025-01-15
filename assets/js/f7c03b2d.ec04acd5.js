"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[7198],{4990:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>d,contentTitle:()=>i,default:()=>h,frontMatter:()=>a,metadata:()=>s,toc:()=>c});const s=JSON.parse('{"id":"detectors/ink/unprotected-set-code-hash","title":"Unprotected set code hash","description":"What it does","source":"@site/docs/detectors/ink/21-unprotected-set-code-hash.md","sourceDirName":"detectors/ink","slug":"/detectors/ink/unprotected-set-code-hash","permalink":"/docs/detectors/ink/unprotected-set-code-hash","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/ink/21-unprotected-set-code-hash.md","tags":[],"version":"current","sidebarPosition":21,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Ink! version","permalink":"/docs/detectors/ink/ink-version"},"next":{"title":"Unprotected Mapping Operation","permalink":"/docs/detectors/ink/unprotected-mapping-operation"}}');var r=n(5105),o=n(6755);const a={},i="Unprotected set code hash",d={},c=[{value:"What it does",id:"what-it-does",level:3},{value:"Why is this bad?",id:"why-is-this-bad",level:3},{value:"Example",id:"example",level:3},{value:"Implementation",id:"implementation",level:3}];function l(e){const t={a:"a",code:"code",h1:"h1",h3:"h3",header:"header",p:"p",pre:"pre",...(0,o.R)(),...e.components};return(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(t.header,{children:(0,r.jsx)(t.h1,{id:"unprotected-set-code-hash",children:"Unprotected set code hash"})}),"\n",(0,r.jsx)(t.h3,{id:"what-it-does",children:"What it does"}),"\n",(0,r.jsxs)(t.p,{children:["It warns you if ",(0,r.jsx)(t.code,{children:"set_code_hash"})," function is called without previously checking the address of the caller."]}),"\n",(0,r.jsx)(t.h3,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,r.jsxs)(t.p,{children:["If users are allowed to call ",(0,r.jsx)(t.code,{children:"set_code_hash"}),", they can intentionally modify the contract behaviour, leading to the loss of all associated data/tokens and functionalities given by this contract or by others that depend on it."]}),"\n",(0,r.jsx)(t.h3,{id:"example",children:"Example"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-rust",children:"    #[ink(message)]\n    pub fn update_code(&self, value: [u8; 32]) -> Result<(), Error> {\n        let res = set_code_hash(&value);\n\n        if res.is_err() {\n            return res.map_err(|_| Error::InvalidCodeHash);\n        }\n\n        Ok(())\n    }\n"})}),"\n",(0,r.jsx)(t.p,{children:"Use instead:"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-rust",children:"    pub fn update_code(&self, value: [u8; 32]) -> Result<(), Error> {\n        if self.admin != Self::env().caller() {\n            return Err(Error::NotAnAdmin);\n        }\n\n        let res = set_code_hash(&value);\n\n        if res.is_err() {\n            return res.map_err(|_| Error::InvalidCodeHash);\n        }\n\n        Ok(())\n    }\n"})}),"\n",(0,r.jsx)(t.h3,{id:"implementation",children:"Implementation"}),"\n",(0,r.jsxs)(t.p,{children:["The detector's implementation can be found at ",(0,r.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout/tree/main/detectors/set-code-hash",children:"this link"})]})]})}function h(e={}){const{wrapper:t}={...(0,o.R)(),...e.components};return t?(0,r.jsx)(t,{...e,children:(0,r.jsx)(l,{...e})}):l(e)}},6755:(e,t,n)=>{n.d(t,{R:()=>a,x:()=>i});var s=n(8101);const r={},o=s.createContext(r);function a(e){const t=s.useContext(o);return s.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function i(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:a(e.components),s.createElement(o.Provider,{value:t},e.children)}}}]);