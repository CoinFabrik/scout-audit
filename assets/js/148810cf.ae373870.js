"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[6576],{8593:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>c,contentTitle:()=>s,default:()=>p,frontMatter:()=>o,metadata:()=>r,toc:()=>l});const r=JSON.parse('{"id":"detectors/ink/unprotected-mapping-operation","title":"Unprotected Mapping Operation","description":"What it does","source":"@site/docs/detectors/ink/22-unprotected-mapping-operation.md","sourceDirName":"detectors/ink","slug":"/detectors/ink/unprotected-mapping-operation","permalink":"/docs/detectors/ink/unprotected-mapping-operation","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/ink/22-unprotected-mapping-operation.md","tags":[],"version":"current","sidebarPosition":22,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Unprotected set code hash","permalink":"/docs/detectors/ink/unprotected-set-code-hash"},"next":{"title":"Lazy storage on delegate","permalink":"/docs/detectors/ink/lazy-delegate"}}');var a=t(5105),i=t(6755);const o={},s="Unprotected Mapping Operation",c={},l=[{value:"What it does",id:"what-it-does",level:3},{value:"Why is this bad?",id:"why-is-this-bad",level:3},{value:"Example",id:"example",level:3},{value:"Implementation",id:"implementation",level:3}];function d(e){const n={a:"a",code:"code",h1:"h1",h3:"h3",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,i.R)(),...e.components};return(0,a.jsxs)(a.Fragment,{children:[(0,a.jsx)(n.header,{children:(0,a.jsx)(n.h1,{id:"unprotected-mapping-operation",children:"Unprotected Mapping Operation"})}),"\n",(0,a.jsx)(n.h3,{id:"what-it-does",children:"What it does"}),"\n",(0,a.jsxs)(n.p,{children:["It warns you if a mapping operation (",(0,a.jsx)(n.code,{children:"insert"}),", ",(0,a.jsx)(n.code,{children:"take"}),", ",(0,a.jsx)(n.code,{children:"remove"}),") function is called with a user-given ",(0,a.jsx)(n.code,{children:"key"})," field of the type ",(0,a.jsx)(n.code,{children:"AccountId"}),"."]}),"\n",(0,a.jsx)(n.h3,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,a.jsx)(n.p,{children:"Modifying mappings with an arbitrary key given by users can be a significant Issue for several reasons:"}),"\n",(0,a.jsxs)(n.ul,{children:["\n",(0,a.jsxs)(n.li,{children:["\n",(0,a.jsx)(n.p,{children:"Unintended Modifications: Allowing users to provide arbitrary keys can lead to unintended modifications of critical data within the smart contract. If the input validation and sanitization are not done properly, users may be able to manipulate the data in ways that were not intended by the contract's author."}),"\n"]}),"\n",(0,a.jsxs)(n.li,{children:["\n",(0,a.jsx)(n.p,{children:"Data Corruption: Malicious users could intentionally provide keys that result in the corruption or manipulation of important data stored in the mapping. This could lead to incorrect calculations, unauthorized access, or other undesirable outcomes."}),"\n"]}),"\n",(0,a.jsxs)(n.li,{children:["\n",(0,a.jsx)(n.p,{children:"Denial-of-Service (DoS) Attacks: If users can set arbitrary keys, they may be able to create mappings with a large number of entries, potentially causing the contract to exceed its gas limit. This could lead to denial-of-service attacks, making the contract unusable for other users."}),"\n"]}),"\n"]}),"\n",(0,a.jsx)(n.h3,{id:"example",children:"Example"}),"\n",(0,a.jsx)(n.pre,{children:(0,a.jsx)(n.code,{className:"language-rust",children:"    #[ink(message)]\n    pub fn withdraw(&mut self, amount: Balance, from: AccountId) -> Result<(), Error> {\n        let current_bal = self.balances.take(from).unwrap_or(0);\n        if current_bal >= amount {\n            self.balances.insert(from, &(current_bal - amount));\n            self.env()\n                .transfer(from, current_bal)\n                .map_err(|_| Error::TransferError)\n        } else {\n            Err(Error::BalanceNotEnough)\n        }\n    }\n"})}),"\n",(0,a.jsx)(n.p,{children:"Use instead:"}),"\n",(0,a.jsx)(n.pre,{children:(0,a.jsx)(n.code,{className:"language-rust",children:"    #[ink(message)]\n    pub fn withdraw(&mut self, amount: Balance) -> Result<(), Error> {\n        let caller = self.env().caller();\n        let current_bal = self.balances.take(caller).unwrap_or(0);\n        if current_bal >= amount {\n            self.balances.insert(caller, &(current_bal - amount));\n            self.env()\n                .transfer(caller, current_bal)\n                .map_err(|_| Error::TransferError)\n        } else {\n            Err(Error::BalanceNotEnough)\n        }\n    }\n"})}),"\n",(0,a.jsx)(n.h3,{id:"implementation",children:"Implementation"}),"\n",(0,a.jsxs)(n.p,{children:["The detector's implementation can be found at ",(0,a.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout/tree/main/detectors/unprotected-mapping-operation",children:"this link"}),"."]})]})}function p(e={}){const{wrapper:n}={...(0,i.R)(),...e.components};return n?(0,a.jsx)(n,{...e,children:(0,a.jsx)(d,{...e})}):d(e)}},6755:(e,n,t)=>{t.d(n,{R:()=>o,x:()=>s});var r=t(8101);const a={},i=r.createContext(a);function o(e){const n=r.useContext(i);return r.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function s(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(a):e.components||a:o(e.components),r.createElement(i.Provider,{value:n},e.children)}}}]);