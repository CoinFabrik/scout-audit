"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[7959],{1134:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>o,contentTitle:()=>s,default:()=>u,frontMatter:()=>c,metadata:()=>a,toc:()=>i});const a=JSON.parse('{"id":"detectors/ink/reentrancy","title":"Reentrancy","description":"What it does","source":"@site/docs/detectors/ink/3-reentrancy.md","sourceDirName":"detectors/ink","slug":"/detectors/ink/reentrancy","permalink":"/docs/detectors/ink/reentrancy","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/ink/3-reentrancy.md","tags":[],"version":"current","sidebarPosition":3,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Set contract storage","permalink":"/docs/detectors/ink/set-contract-storage"},"next":{"title":"Panic error","permalink":"/docs/detectors/ink/panic-error"}}');var r=t(5105),l=t(6755);const c={},s="Reentrancy",o={},i=[{value:"What it does",id:"what-it-does",level:3},{value:"Why is this bad?",id:"why-is-this-bad",level:3},{value:"Known problems",id:"known-problems",level:3},{value:"Example",id:"example",level:3},{value:"Implementation",id:"implementation",level:3}];function d(e){const n={a:"a",code:"code",h1:"h1",h3:"h3",header:"header",p:"p",pre:"pre",...(0,l.R)(),...e.components};return(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(n.header,{children:(0,r.jsx)(n.h1,{id:"reentrancy",children:"Reentrancy"})}),"\n",(0,r.jsx)(n.h3,{id:"what-it-does",children:"What it does"}),"\n",(0,r.jsx)(n.p,{children:"This linting rule checks whether the 'check-effects-interaction' pattern has been properly followed by any code that invokes a contract that may call back to the original one."}),"\n",(0,r.jsx)(n.h3,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,r.jsx)(n.p,{children:"If state modifications are made after a contract call, reentrant calls may not detect these modifications, potentially leading to unexpected behaviors such as double spending."}),"\n",(0,r.jsx)(n.h3,{id:"known-problems",children:"Known problems"}),"\n",(0,r.jsxs)(n.p,{children:["If called method does not perform a malicious reentrancy (i.e. known method from known contract) false positives will arise.\nIf the usage of ",(0,r.jsx)(n.code,{children:"set_allow_reentry(true)"})," or later state changes are performed in an auxiliary function, this detector will not detect the reentrancy."]}),"\n",(0,r.jsx)(n.h3,{id:"example",children:"Example"}),"\n",(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-rust",children:"let caller_addr = self.env().caller();\nlet caller_balance = self.balance(caller_addr);\n\nif amount > caller_balance {\n    return Ok(caller_balance);\n}\n\nlet call = build_call::<ink::env::DefaultEnvironment>()\n    .call(address)\n    .transferred_value(amount)\n    .exec_input(ink::env::call::ExecutionInput::new(Selector::new(\n        selector.to_be_bytes(),\n    )))\n    .call_flags(ink::env::CallFlags::default().set_allow_reentry(true))\n    .returns::<()>()\n    .params();\nself.env()\n    .invoke_contract(&call)\n    .map_err(|_| Error::ContractInvokeFailed)?\n    .map_err(|_| Error::ContractInvokeFailed)?;\n\nlet new_balance = caller_balance.checked_sub(amount).ok_or(Error::Underflow)?;\nself.balances.insert(caller_addr, &new_balance);\n"})}),"\n",(0,r.jsx)(n.p,{children:"Use instead:"}),"\n",(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-rust",children:'let caller_addr = self.env().caller();\nlet caller_balance = self.balances.get(caller_addr).unwrap_or(0);\nif amount <= caller_balance {\n    //The balance is updated before the contract call\n    self.balances\n        .insert(caller_addr, &(caller_balance - amount));\n    let call = build_call::<ink::env::DefaultEnvironment>()\n        .call(address)\n        .transferred_value(amount)\n        .exec_input(ink::env::call::ExecutionInput::new(Selector::new(\n            selector.to_be_bytes(),\n        )))\n        .call_flags(ink::env::CallFlags::default().set_allow_reentry(true))\n        .returns::<()>()\n        .params();\n    self.env()\n        .invoke_contract(&call)\n        .unwrap_or_else(|err| panic!("Err {:?}", err))\n        .unwrap_or_else(|err| panic!("LangErr {:?}", err));\n\n    return caller_balance - amount;\n} else {\n    return caller_balance;\n}\n'})}),"\n",(0,r.jsx)(n.h3,{id:"implementation",children:"Implementation"}),"\n",(0,r.jsxs)(n.p,{children:["The detector's implementation can be found at these links ",(0,r.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout/tree/main/detectors/reentrancy-1",children:"link1"}),", ",(0,r.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout/tree/main/detectors/reentrancy-2",children:"link2"}),"."]})]})}function u(e={}){const{wrapper:n}={...(0,l.R)(),...e.components};return n?(0,r.jsx)(n,{...e,children:(0,r.jsx)(d,{...e})}):d(e)}},6755:(e,n,t)=>{t.d(n,{R:()=>c,x:()=>s});var a=t(8101);const r={},l=a.createContext(r);function c(e){const n=a.useContext(l);return a.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function s(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:c(e.components),a.createElement(l.Provider,{value:n},e.children)}}}]);