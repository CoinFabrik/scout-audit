"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[234],{3313:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>c,contentTitle:()=>r,default:()=>p,frontMatter:()=>o,metadata:()=>s,toc:()=>d});const s=JSON.parse('{"id":"detectors/soroban/unprotected-mapping-operation","title":"Unprotected mapping operation","description":"Description","source":"@site/docs/detectors/soroban/16-unprotected-mapping-operation.md","sourceDirName":"detectors/soroban","slug":"/detectors/soroban/unprotected-mapping-operation","permalink":"/docs/detectors/soroban/unprotected-mapping-operation","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/soroban/16-unprotected-mapping-operation.md","tags":[],"version":"current","sidebarPosition":16,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Assert violation","permalink":"/docs/detectors/soroban/assert-violation"},"next":{"title":"DoS unexpected revert with vector","permalink":"/docs/detectors/soroban/dos-unexpected-revert-with-vector"}}');var a=n(5105),i=n(6755);const o={},r="Unprotected mapping operation",c={},d=[{value:"Description",id:"description",level:2},{value:"Why is this bad?",id:"why-is-this-bad",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediated example",id:"remediated-example",level:2},{value:"How is it detected?",id:"how-is-it-detected",level:2}];function l(e){const t={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,i.R)(),...e.components};return(0,a.jsxs)(a.Fragment,{children:[(0,a.jsx)(t.header,{children:(0,a.jsx)(t.h1,{id:"unprotected-mapping-operation",children:"Unprotected mapping operation"})}),"\n",(0,a.jsx)(t.h2,{id:"description",children:"Description"}),"\n",(0,a.jsxs)(t.ul,{children:["\n",(0,a.jsxs)(t.li,{children:["Category: ",(0,a.jsx)(t.code,{children:"Authorization"})]}),"\n",(0,a.jsxs)(t.li,{children:["Severity: ",(0,a.jsx)(t.code,{children:"Critical"})]}),"\n",(0,a.jsxs)(t.li,{children:["Detector: ",(0,a.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unprotected-mapping-operation",children:(0,a.jsx)(t.code,{children:"unprotected-mapping-operation"})})]}),"\n",(0,a.jsxs)(t.li,{children:["Test Cases: ",(0,a.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-mapping-operation/unprotected-mapping-operation-1",children:(0,a.jsx)(t.code,{children:"unprotected-mapping-operation-1"})})," ",(0,a.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-mapping-operation/unprotected-mapping-operation-2",children:(0,a.jsx)(t.code,{children:"unprotected-mapping-operation-2"})})]}),"\n"]}),"\n",(0,a.jsx)(t.p,{children:"In Rust, Modifying mappings with an arbitrary key given by the user could lead to several issues. Ideally, only users who have been previously verified should be able to do it."}),"\n",(0,a.jsx)(t.h2,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,a.jsxs)(t.ul,{children:["\n",(0,a.jsxs)(t.li,{children:["\n",(0,a.jsx)(t.p,{children:"Unintended Modifications: Allowing users to provide arbitrary keys can lead to unintended modifications of critical data within the smart contract. If the input validation and sanitation are not done properly, users may be able to manipulate the data in ways that were not intended by the contract's author."}),"\n"]}),"\n",(0,a.jsxs)(t.li,{children:["\n",(0,a.jsx)(t.p,{children:"Data Corruption: Malicious users could intentionally provide keys that result in the corruption or manipulation of important data stored in the mapping. This could lead to incorrect calculations, unauthorized access, or other undesirable outcomes."}),"\n"]}),"\n",(0,a.jsxs)(t.li,{children:["\n",(0,a.jsx)(t.p,{children:"Denial-of-Service (DoS) Attacks: If users can set arbitrary keys, they may be able to create mappings with a large number of entries, potentially causing the contract to exceed its gas limit. This could lead to denial-of-service attacks, making the contract unusable for other users."}),"\n"]}),"\n"]}),"\n",(0,a.jsx)(t.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,a.jsxs)(t.p,{children:["Consider the following ",(0,a.jsx)(t.code,{children:"Soroban"})," contract:"]}),"\n",(0,a.jsx)(t.pre,{children:(0,a.jsx)(t.code,{className:"language-rust",children:"   pub fn set_balance(env: Env, address: Address, balance: i128) -> State {\n        // Get the current state.\n        let mut state = Self::get_state(env.clone());\n\n        // Set the new account to have total supply if it doesn't exist.\n        if !state.balances.contains_key(address.clone()) {\n            state.balances.set(address, balance);\n            // Save the state.\n            env.storage().persistent().set(&STATE, &state);\n        }\n\n        state\n    }\n"})}),"\n",(0,a.jsxs)(t.p,{children:["The ",(0,a.jsx)(t.code,{children:"set_balance()"})," function allows anyone to call it and modify the account balances in the state. It lacks authorization checks and allows modifying the mutable state directly."]}),"\n",(0,a.jsxs)(t.p,{children:["The code example can be found ",(0,a.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-mapping-operation/unprotected-mapping-operation-1/vulnerable-example",children:"here"}),"."]}),"\n",(0,a.jsx)(t.h2,{id:"remediated-example",children:"Remediated example"}),"\n",(0,a.jsxs)(t.p,{children:["The fix adds an ",(0,a.jsx)(t.code,{children:"address.require_auth()"})," step, likely checking user permissions to update balances. This ensures only authorized users can modify account data."]}),"\n",(0,a.jsx)(t.pre,{children:(0,a.jsx)(t.code,{className:"language-rust",children:"    pub fn set_balance(env: Env, address: Address, balance: i128) -> State {\n        // Authenticate user\n        address.require_auth();\n\n        // Get the current state.\n        let mut state = Self::get_state(env.clone());\n\n        // Set the new account to have total supply if it doesn't exist.\n        if !state.balances.contains_key(address.clone()) {\n            state.balances.set(address, balance);\n            // Save the state.\n            env.storage().persistent().set(&STATE, &state);\n        }\n\n        state\n    }\n"})}),"\n",(0,a.jsxs)(t.p,{children:["The remediated code example can be found ",(0,a.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-mapping-operation/unprotected-mapping-operation-1/remediated-example",children:(0,a.jsx)(t.code,{children:"here"})}),"."]}),"\n",(0,a.jsx)(t.h2,{id:"how-is-it-detected",children:"How is it detected?"}),"\n",(0,a.jsxs)(t.p,{children:["It warns you if a mapping operation (",(0,a.jsx)(t.code,{children:"insert"}),", ",(0,a.jsx)(t.code,{children:"take"}),", ",(0,a.jsx)(t.code,{children:"remove"}),") function is called with a user-given ",(0,a.jsx)(t.code,{children:"key"})," field of the type ",(0,a.jsx)(t.code,{children:"AccountId"}),"."]})]})}function p(e={}){const{wrapper:t}={...(0,i.R)(),...e.components};return t?(0,a.jsx)(t,{...e,children:(0,a.jsx)(l,{...e})}):l(e)}},6755:(e,t,n)=>{n.d(t,{R:()=>o,x:()=>r});var s=n(8101);const a={},i=s.createContext(a);function o(e){const t=s.useContext(i);return s.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function r(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(a):e.components||a:o(e.components),s.createElement(i.Provider,{value:t},e.children)}}}]);