"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[9043],{8130:(e,t,r)=>{r.r(t),r.d(t,{assets:()=>d,contentTitle:()=>i,default:()=>m,frontMatter:()=>a,metadata:()=>n,toc:()=>c});const n=JSON.parse('{"id":"detectors/ink/avoid-format-string","title":"Avoid fromat! macro usage","description":"What it does","source":"@site/docs/detectors/ink/17-avoid-format-string.md","sourceDirName":"detectors/ink","slug":"/detectors/ink/avoid-format-string","permalink":"/docs/detectors/ink/avoid-format-string","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/ink/17-avoid-format-string.md","tags":[],"version":"current","sidebarPosition":17,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Avoid core::mem::forget usage","permalink":"/docs/detectors/ink/avoid-core-mem-forget"},"next":{"title":"Unprotected self destruct","permalink":"/docs/detectors/ink/unprotected-self-destruct"}}');var s=r(5105),o=r(6755);const a={},i="Avoid fromat! macro usage",d={},c=[{value:"What it does",id:"what-it-does",level:3},{value:"Why is this bad?",id:"why-is-this-bad",level:3},{value:"Example",id:"example",level:3},{value:"Implementation",id:"implementation",level:3}];function l(e){const t={a:"a",code:"code",h1:"h1",h3:"h3",header:"header",p:"p",pre:"pre",...(0,o.R)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(t.header,{children:(0,s.jsx)(t.h1,{id:"avoid-fromat-macro-usage",children:"Avoid fromat! macro usage"})}),"\n",(0,s.jsx)(t.h3,{id:"what-it-does",children:"What it does"}),"\n",(0,s.jsxs)(t.p,{children:["Checks for ",(0,s.jsx)(t.code,{children:"format!"})," macro usage."]}),"\n",(0,s.jsx)(t.h3,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,s.jsxs)(t.p,{children:["The usage of ",(0,s.jsx)(t.code,{children:"format!"})," is not recommended."]}),"\n",(0,s.jsx)(t.h3,{id:"example",children:"Example"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:'    #[ink(message)]\n    pub fn crash(&self) -> Result<(), Error> {\n        Err(Error::FormatError {\n            msg: (format!("{}", self.value)),\n        })\n    }\n'})}),"\n",(0,s.jsx)(t.p,{children:"Use instead:"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:"    pub enum Error {\n        FormatError { msg: String },\n        CrashError\n    }\n\n    #[ink(message)]\n    pub fn crash(&self) -> Result<(), Error> {\n        Err(Error::FormatError { msg: self.value.to_string() })\n    }\n"})}),"\n",(0,s.jsx)(t.h3,{id:"implementation",children:"Implementation"}),"\n",(0,s.jsxs)(t.p,{children:["The detector's implementation can be found at ",(0,s.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout/tree/main/detectors/avoid-format-string",children:"this link"}),"."]})]})}function m(e={}){const{wrapper:t}={...(0,o.R)(),...e.components};return t?(0,s.jsx)(t,{...e,children:(0,s.jsx)(l,{...e})}):l(e)}},6755:(e,t,r)=>{r.d(t,{R:()=>a,x:()=>i});var n=r(8101);const s={},o=n.createContext(s);function a(e){const t=n.useContext(o);return n.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function i(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:a(e.components),n.createElement(o.Provider,{value:t},e.children)}}}]);