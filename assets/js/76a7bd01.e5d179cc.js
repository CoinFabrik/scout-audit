"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[711],{6058:(e,r,o)=>{o.r(r),o.d(r,{assets:()=>a,contentTitle:()=>l,default:()=>u,frontMatter:()=>i,metadata:()=>n,toc:()=>d});const n=JSON.parse('{"id":"detectors/soroban/integer-overflow -or-underflow","title":"Integer overflow or underflow","description":"Description","source":"@site/docs/detectors/soroban/21-integer-overflow -or-underflow.md","sourceDirName":"detectors/soroban","slug":"/detectors/soroban/integer-overflow -or-underflow","permalink":"/docs/detectors/soroban/integer-overflow -or-underflow","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/soroban/21-integer-overflow -or-underflow.md","tags":[],"version":"current","sidebarPosition":21,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Incorrect exponentiation","permalink":"/docs/detectors/soroban/incorrect-exponentiation"},"next":{"title":"Storage change events","permalink":"/docs/detectors/soroban/storage-change-events"}}');var t=o(5105),s=o(6755);const i={},l="Integer overflow or underflow",a={},d=[{value:"Description",id:"description",level:2},{value:"Why is this bad?",id:"why-is-this-bad",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediated example",id:"remediated-example",level:2},{value:"How is it detected?",id:"how-is-it-detected",level:2}];function c(e){const r={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,s.R)(),...e.components};return(0,t.jsxs)(t.Fragment,{children:[(0,t.jsx)(r.header,{children:(0,t.jsx)(r.h1,{id:"integer-overflow-or-underflow",children:"Integer overflow or underflow"})}),"\n",(0,t.jsx)(r.h2,{id:"description",children:"Description"}),"\n",(0,t.jsxs)(r.ul,{children:["\n",(0,t.jsxs)(r.li,{children:["Category: ",(0,t.jsx)(r.code,{children:"Arithmetic"})]}),"\n",(0,t.jsxs)(r.li,{children:["Severity: ",(0,t.jsx)(r.code,{children:"Critical"})]}),"\n",(0,t.jsxs)(r.li,{children:["Detectors: ",(0,t.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/integer-overflow-or-underflow",children:(0,t.jsx)(r.code,{children:"integer-overflow-or-underflow"})})]}),"\n",(0,t.jsxs)(r.li,{children:["Test Cases: ",(0,t.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-1",children:(0,t.jsx)(r.code,{children:"integer-overflow-or-underflow-1"})}),"\n",(0,t.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-2",children:(0,t.jsx)(r.code,{children:"integer-overflow-or-underflow-2"})}),"\n",(0,t.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-3",children:(0,t.jsx)(r.code,{children:"integer-overflow-or-underflow-3"})}),"\n",(0,t.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-4",children:(0,t.jsx)(r.code,{children:"integer-overflow-or-underflow-4"})}),"\n",(0,t.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-5",children:(0,t.jsx)(r.code,{children:"integer-overflow-or-underflow-5"})})]}),"\n"]}),"\n",(0,t.jsx)(r.p,{children:"In Rust, arithmetic operations can result in a value that falls outside the allowed numerical range for a given type. When the result exceeds the maximum value of the range, it's called an overflow, and when it falls below the minimum value of the range, it's called an underflow."}),"\n",(0,t.jsx)(r.h2,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,t.jsx)(r.p,{children:"If there are arithmetic operations with overflow or underflow problems, and if errors are not handled correctly, incorrect results will be generated, bringing potential problems for the contract. Additionally, these types of errors can allow attackers to drain a contract\u2019s funds or manipulate its logic."}),"\n",(0,t.jsx)(r.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,t.jsxs)(r.p,{children:["Consider the following ",(0,t.jsx)(r.code,{children:"Soroban"})," contract:"]}),"\n",(0,t.jsx)(r.pre,{children:(0,t.jsx)(r.code,{className:"language-rust",children:"\n pub fn add(env: Env, value: u32) {\n        let current: u32 = env.storage().temporary().get(&Self::VALUE).unwrap_or(0);\n        let new_value = current + value;\n        env.storage().temporary().set(&Self::VALUE, &new_value);\n    }\n\n"})}),"\n",(0,t.jsx)(r.p,{children:"In this example, an operation is performed on two u32 values without any safeguards against overflow if it occurs."}),"\n",(0,t.jsxs)(r.p,{children:["The code example can be found ",(0,t.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-1/vulnerable-example",children:"here"}),"."]}),"\n",(0,t.jsx)(r.h2,{id:"remediated-example",children:"Remediated example"}),"\n",(0,t.jsx)(r.pre,{children:(0,t.jsx)(r.code,{className:"language-rust",children:"pub fn add(env: Env, value: u32) -> Result<(), Error> {\n        let current: u32 = env.storage().temporary().get(&Self::VALUE).unwrap_or(0);\n        let new_value = match current.checked_add(value) {\n            Some(value) => value,\n            None => return Err(Error::OverflowError),\n        };\n        env.storage().temporary().set(&Self::VALUE, &new_value);\n        Ok(())\n    }       \n"})}),"\n",(0,t.jsxs)(r.p,{children:["In this example, the ",(0,t.jsx)(r.code,{children:"checked_add"})," method is used to perform the addition. It returns the sum if no overflow occurs; otherwise, it returns ",(0,t.jsx)(r.code,{children:"None"}),", with an OverflowError variant indicating that an overflow error has occurred."]}),"\n",(0,t.jsxs)(r.p,{children:["The remediated code example can be found ",(0,t.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-1/remediated-example",children:"here"}),"."]}),"\n",(0,t.jsx)(r.h2,{id:"how-is-it-detected",children:"How is it detected?"}),"\n",(0,t.jsx)(r.p,{children:"Checks if there\u2019s any numerical overflow or underflow."})]})}function u(e={}){const{wrapper:r}={...(0,s.R)(),...e.components};return r?(0,t.jsx)(r,{...e,children:(0,t.jsx)(c,{...e})}):c(e)}},6755:(e,r,o)=>{o.d(r,{R:()=>i,x:()=>l});var n=o(8101);const t={},s=n.createContext(t);function i(e){const r=n.useContext(s);return n.useMemo((function(){return"function"==typeof e?e(r):{...r,...e}}),[r,e])}function l(e){let r;return r=e.disableParentContext?"function"==typeof e.components?e.components(t):e.components||t:i(e.components),n.createElement(s.Provider,{value:r},e.children)}}}]);