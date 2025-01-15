"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[3880],{8403:(e,n,r)=>{r.r(n),r.d(n,{assets:()=>d,contentTitle:()=>o,default:()=>u,frontMatter:()=>i,metadata:()=>t,toc:()=>c});const t=JSON.parse('{"id":"detectors/ink/unsafe-unwrap","title":"Unsafe unwrap","description":"What it does","source":"@site/docs/detectors/ink/9-unsafe-unwrap.md","sourceDirName":"detectors/ink","slug":"/detectors/ink/unsafe-unwrap","permalink":"/docs/detectors/ink/unsafe-unwrap","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/ink/9-unsafe-unwrap.md","tags":[],"version":"current","sidebarPosition":9,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Unsafe expect","permalink":"/docs/detectors/ink/unsafe-expect"},"next":{"title":"Divide before multiply","permalink":"/docs/detectors/ink/divide-before-multiply"}}');var s=r(5105),a=r(6755);const i={},o="Unsafe unwrap",d={},c=[{value:"What it does",id:"what-it-does",level:3},{value:"Why is this bad?",id:"why-is-this-bad",level:3},{value:"Example",id:"example",level:3},{value:"Implementation",id:"implementation",level:3}];function l(e){const n={a:"a",code:"code",h1:"h1",h3:"h3",header:"header",p:"p",pre:"pre",...(0,a.R)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(n.header,{children:(0,s.jsx)(n.h1,{id:"unsafe-unwrap",children:"Unsafe unwrap"})}),"\n",(0,s.jsx)(n.h3,{id:"what-it-does",children:"What it does"}),"\n",(0,s.jsxs)(n.p,{children:["Checks for usage of ",(0,s.jsx)(n.code,{children:".unwrap()"})]}),"\n",(0,s.jsx)(n.h3,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,s.jsxs)(n.p,{children:[(0,s.jsx)(n.code,{children:".unwrap()"})," might panic if the result value is an error or ",(0,s.jsx)(n.code,{children:"None"}),"."]}),"\n",(0,s.jsx)(n.h3,{id:"example",children:"Example"}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-rust",children:'// example code where a warning is issued\nfn main() {\n    let result = result_fn().unwrap("error");\n}\nfn result_fn() -> Result<u8, Error> {\n    Err(Error::new(ErrorKind::Other, "error"))\n}\n'})}),"\n",(0,s.jsx)(n.p,{children:"Use instead:"}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-rust",children:'// example code that does not raise a warning\nfn main() {\n   let result = if let Ok(result) = result_fn() {\n      result\n  }\n}\nfn result_fn() -> Result<u8, Error> {\n    Err(Error::new(ErrorKind::Other, "error"))\n}\n'})}),"\n",(0,s.jsx)(n.h3,{id:"implementation",children:"Implementation"}),"\n",(0,s.jsxs)(n.p,{children:["The detector's implementation can be found at ",(0,s.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout/tree/main/detectors/unsafe-unwrap",children:"this link"}),"."]})]})}function u(e={}){const{wrapper:n}={...(0,a.R)(),...e.components};return n?(0,s.jsx)(n,{...e,children:(0,s.jsx)(l,{...e})}):l(e)}},6755:(e,n,r)=>{r.d(n,{R:()=>i,x:()=>o});var t=r(8101);const s={},a=t.createContext(s);function i(e){const n=t.useContext(a);return t.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function o(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:i(e.components),t.createElement(a.Provider,{value:n},e.children)}}}]);