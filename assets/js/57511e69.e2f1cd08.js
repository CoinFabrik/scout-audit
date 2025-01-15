"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[516],{5793:(e,t,r)=>{r.r(t),r.d(t,{assets:()=>c,contentTitle:()=>i,default:()=>h,frontMatter:()=>a,metadata:()=>n,toc:()=>d});const n=JSON.parse('{"id":"detectors/soroban/unrestricted-transfer-from","title":"Unrestricted transfer from","description":"Description","source":"@site/docs/detectors/soroban/18-unrestricted-transfer-from.md","sourceDirName":"detectors/soroban","slug":"/detectors/soroban/unrestricted-transfer-from","permalink":"/docs/detectors/soroban/unrestricted-transfer-from","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/soroban/18-unrestricted-transfer-from.md","tags":[],"version":"current","sidebarPosition":18,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"DoS unexpected revert with vector","permalink":"/docs/detectors/soroban/dos-unexpected-revert-with-vector"},"next":{"title":"Unsafe map get","permalink":"/docs/detectors/soroban/unsafe-map-get"}}');var s=r(5105),o=r(6755);const a={},i="Unrestricted transfer from",c={},d=[{value:"Description",id:"description",level:2},{value:"Why is this bad?",id:"why-is-this-bad",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediated example",id:"remediated-example",level:2},{value:"How is it detected?",id:"how-is-it-detected",level:2},{value:"References",id:"references",level:2}];function l(e){const t={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,o.R)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(t.header,{children:(0,s.jsx)(t.h1,{id:"unrestricted-transfer-from",children:"Unrestricted transfer from"})}),"\n",(0,s.jsx)(t.h2,{id:"description",children:"Description"}),"\n",(0,s.jsxs)(t.ul,{children:["\n",(0,s.jsxs)(t.li,{children:["Category: ",(0,s.jsx)(t.code,{children:"Validations and error handling"})]}),"\n",(0,s.jsxs)(t.li,{children:["Severity: ",(0,s.jsx)(t.code,{children:"High"})]}),"\n",(0,s.jsxs)(t.li,{children:["Detectors: ",(0,s.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unrestricted-transfer-from",children:(0,s.jsx)(t.code,{children:"unrestricted-transfer-from"})})]}),"\n",(0,s.jsxs)(t.li,{children:["Test Cases: ",(0,s.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unrestricted-transfer-from/unrestricted-transfer-from-1",children:(0,s.jsx)(t.code,{children:"unrestricted-transfer-from-1"})})]}),"\n"]}),"\n",(0,s.jsxs)(t.p,{children:["Allowing unrestricted ",(0,s.jsx)(t.code,{children:"transfer_from"})," operations poses a significant issue. When ",(0,s.jsx)(t.code,{children:"from"})," arguments for that function is provided directly by the user, this might enable the withdrawal of funds from any actor with token approval on the contract."]}),"\n",(0,s.jsx)(t.h2,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,s.jsxs)(t.p,{children:["The absence of proper authorization checks for sensitive operations, like ",(0,s.jsx)(t.code,{children:"transfer_from"}),", can lead to the loss of funds or other undesired consequences. For example, if a user, Alice, approves a contract to spend her tokens, and the contract lacks proper authorization checks, another user, Bob, could invoke the contract and potentially transfer Alice's tokens to himself without her explicit consent."]}),"\n",(0,s.jsx)(t.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,s.jsxs)(t.p,{children:["Consider the following ",(0,s.jsx)(t.code,{children:"Soroban"})," function:"]}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:"     pub fn deposit(env: Env, from: Address) -> Result<(), UTFError> {\n        let mut state: State = Self::get_state(env.clone())?;\n        state.buyer.require_auth();\n        if state.status != Status::Created {\n            return Err(UTFError::StatusMustBeCreated);\n        }\n        let token_client = token::Client::new(&env, &state.token);\n        token_client.transfer_from(\n            &env.current_contract_address(),\n            &from,\n            &env.current_contract_address(),\n            &state.amount,\n        );\n        state.status = Status::Locked;\n        env.storage().instance().set(&STATE, &state);\n        Ok(())\n    }\n"})}),"\n",(0,s.jsxs)(t.p,{children:["The issue in this ",(0,s.jsx)(t.code,{children:"deposit"})," function arises from the use of ",(0,s.jsx)(t.code,{children:"from"}),", an user-defined parameter as an argument in the ",(0,s.jsx)(t.code,{children:"from"})," field of the ",(0,s.jsx)(t.code,{children:"transfer_from"})," function. Alice can approve a contract to spend their tokens, then Bob can call that contract, use that allowance to send as themselves Alice's tokens."]}),"\n",(0,s.jsxs)(t.p,{children:["The code example can be found ",(0,s.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unrestricted-transfer-from/unrestricted-transfer-from-1/vulnerable-example",children:(0,s.jsx)(t.code,{children:"here"})}),"."]}),"\n",(0,s.jsx)(t.h2,{id:"remediated-example",children:"Remediated example"}),"\n",(0,s.jsxs)(t.p,{children:["Avoid using user-defined arguments as ",(0,s.jsx)(t.code,{children:"from"})," parameter in ",(0,s.jsx)(t.code,{children:"transfer_from"}),". Instead, use ",(0,s.jsx)(t.code,{children:"state.buyer"})," as shown in the following example."]}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:"     pub fn deposit(env: Env) -> Result<(), UTFError> {\n        let mut state: State = Self::get_state(env.clone())?;\n        state.buyer.require_auth();\n        if state.status != Status::Created {\n            return Err(UTFError::StatusMustBeCreated);\n        }\n        let token_client = token::Client::new(&env, &state.token);\n        token_client.transfer_from(\n            &env.current_contract_address(),\n            &state.buyer,\n            &env.current_contract_address(),\n            &state.amount,\n        );\n        state.status = Status::Locked;\n        env.storage().instance().set(&STATE, &state);\n        Ok(())\n    }\n"})}),"\n",(0,s.jsxs)(t.p,{children:["The remediated code example can be found ",(0,s.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unrestricted-transfer-from/unrestricted-transfer-from-1/remediated-example",children:"here"}),"."]}),"\n",(0,s.jsx)(t.h2,{id:"how-is-it-detected",children:"How is it detected?"}),"\n",(0,s.jsxs)(t.p,{children:["It warns you if a ",(0,s.jsx)(t.code,{children:"transfer_from"})," function is called with a user-defined parameter in the ",(0,s.jsx)(t.code,{children:"from"})," field."]}),"\n",(0,s.jsx)(t.h2,{id:"references",children:"References"}),"\n",(0,s.jsxs)(t.ul,{children:["\n",(0,s.jsx)(t.li,{children:(0,s.jsx)(t.a,{href:"https://github.com/crytic/slither/wiki/Detector-Documentation#arbitrary-from-in-transferfrom",children:"Slither: Arbitrary from in transferFrom"})}),"\n"]})]})}function h(e={}){const{wrapper:t}={...(0,o.R)(),...e.components};return t?(0,s.jsx)(t,{...e,children:(0,s.jsx)(l,{...e})}):l(e)}},6755:(e,t,r)=>{r.d(t,{R:()=>a,x:()=>i});var n=r(8101);const s={},o=n.createContext(s);function a(e){const t=n.useContext(o);return n.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function i(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:a(e.components),n.createElement(o.Provider,{value:t},e.children)}}}]);