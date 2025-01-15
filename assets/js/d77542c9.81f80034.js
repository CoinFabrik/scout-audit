"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[9582],{9498:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>l,contentTitle:()=>i,default:()=>u,frontMatter:()=>o,metadata:()=>s,toc:()=>c});const s=JSON.parse('{"id":"detectors/ink/unrestricted-transfer-from","title":"Unrestricted Transfer From","description":"What it does","source":"@site/docs/detectors/ink/14-unrestricted-transfer-from.md","sourceDirName":"detectors/ink","slug":"/detectors/ink/unrestricted-transfer-from","permalink":"/scout-audit/docs/detectors/ink/unrestricted-transfer-from","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/ink/14-unrestricted-transfer-from.md","tags":[],"version":"current","sidebarPosition":14,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Insuficciently random values","permalink":"/scout-audit/docs/detectors/ink/insufficiently-random-values"},"next":{"title":"Assert violation","permalink":"/scout-audit/docs/detectors/ink/assert-violation"}}');var r=t(5105),a=t(6755);const o={},i="Unrestricted Transfer From",l={},c=[{value:"What it does",id:"what-it-does",level:3},{value:"Why is this bad?",id:"why-is-this-bad",level:3},{value:"Known problems",id:"known-problems",level:3},{value:"Example",id:"example",level:3},{value:"Implementation",id:"implementation",level:3}];function d(e){const n={a:"a",code:"code",h1:"h1",h3:"h3",header:"header",p:"p",pre:"pre",...(0,a.R)(),...e.components};return(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(n.header,{children:(0,r.jsx)(n.h1,{id:"unrestricted-transfer-from",children:"Unrestricted Transfer From"})}),"\n",(0,r.jsx)(n.h3,{id:"what-it-does",children:"What it does"}),"\n",(0,r.jsxs)(n.p,{children:["It warns you if a ",(0,r.jsx)(n.code,{children:"transfer_from"})," function is called with a user-defined parameter in the ",(0,r.jsx)(n.code,{children:"from"})," field."]}),"\n",(0,r.jsx)(n.h3,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,r.jsx)(n.p,{children:"An user Alice can approve a contract to spend their tokens. An user Bob can call that contract, use that allowance to send themselves Alice's tokens."}),"\n",(0,r.jsx)(n.h3,{id:"known-problems",children:"Known problems"}),"\n",(0,r.jsxs)(n.p,{children:["Could generate false positives when using ",(0,r.jsx)(n.a,{href:"https://github.com/Cardinal-Cryptography/PSP22",children:"Cardinal Cryptography's PSP22"}),"."]}),"\n",(0,r.jsx)(n.h3,{id:"example",children:"Example"}),"\n",(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-rust",children:'// build_call example\n    #[ink(message)]\n    pub fn deposit(&mut self, from: AccountId) -> Result<(), Error> {\n        let call_params = build_call::<DefaultEnvironment>()\n            .exec_input(\n                ExecutionInput::new(Selector::new(ink::selector_bytes!(\n                    "PSP22::transfer_from"\n                )))\n                .push_arg(from)\n                .push_arg(self.env().account_id())\n                .push_arg(self.amount)\n                .push_arg([0u8]),\n            )\n    }\n// ContractRef example\n    #[ink(message)]\n    pub fn deposit(&mut self, from: AccountId) -> Result<(), Error> {\n        let res = PSP22Ref::transfer_from(\n            &self.psp22_address,\n            from,\n            self.env().account_id(),\n            self.amount,\n            vec![],\n        );\n    }\n'})}),"\n",(0,r.jsx)(n.p,{children:"Use instead:"}),"\n",(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-rust",children:'// build_call example\n    #[ink(message)]\n    pub fn deposit(&mut self) -> Result<(), Error> {\n        let call_params = build_call::<DefaultEnvironment>()\n            .exec_input(\n                ExecutionInput::new(Selector::new(ink::selector_bytes!(\n                    "PSP22::transfer_from"\n                )))\n                .push_arg(self.env().caller())\n                .push_arg(self.env().account_id())\n                .push_arg(self.amount)\n                .push_arg([0u8]),\n            )\n    }\n\n// ContractRef example\n    #[ink(message)]\n    pub fn deposit(&mut self) -> Result<(), Error> {\n        let res = PSP22Ref::transfer_from(\n            &self.psp22_address,\n            self.env().caller(),\n            self.env().account_id(),\n            self.amount,\n            vec![],\n        );\n    }\n\n'})}),"\n",(0,r.jsx)(n.h3,{id:"implementation",children:"Implementation"}),"\n",(0,r.jsxs)(n.p,{children:["The detector's implementation can be found at ",(0,r.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout/tree/main/detectors/unrestricted-transfer-from",children:"this link"}),"."]})]})}function u(e={}){const{wrapper:n}={...(0,a.R)(),...e.components};return n?(0,r.jsx)(n,{...e,children:(0,r.jsx)(d,{...e})}):d(e)}},6755:(e,n,t)=>{t.d(n,{R:()=>o,x:()=>i});var s=t(8101);const r={},a=s.createContext(r);function o(e){const n=s.useContext(a);return s.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function i(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:o(e.components),s.createElement(a.Provider,{value:n},e.children)}}}]);