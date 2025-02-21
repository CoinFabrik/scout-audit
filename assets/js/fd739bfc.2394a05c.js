"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[8715],{3881:(e,n,t)=>{t.d(n,{R:()=>i,x:()=>s});var r=t(8101);const o={},a=r.createContext(o);function i(e){const n=r.useContext(a);return r.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function s(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(o):e.components||o:i(e.components),r.createElement(a.Provider,{value:n},e.children)}},7416:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>c,contentTitle:()=>s,default:()=>h,frontMatter:()=>i,metadata:()=>r,toc:()=>d});const r=JSON.parse('{"id":"detectors/soroban/avoid-panic-error","title":"Avoid panic error","description":"Description","source":"@site/docs/detectors/soroban/9-avoid-panic-error.md","sourceDirName":"detectors/soroban","slug":"/detectors/soroban/avoid-panic-error","permalink":"/scout-audit/docs/detectors/soroban/avoid-panic-error","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/soroban/9-avoid-panic-error.md","tags":[],"version":"current","sidebarPosition":9,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Set contract storage","permalink":"/scout-audit/docs/detectors/soroban/set-contract-storage"},"next":{"title":"Avoid unsafe block","permalink":"/scout-audit/docs/detectors/soroban/avoid-unsafe-block"}}');var o=t(5105),a=t(3881);const i={},s="Avoid panic error",c={},d=[{value:"Description",id:"description",level:2},{value:"Why is this bad?",id:"why-is-this-bad",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediated example",id:"remediated-example",level:2},{value:"How is it detected?",id:"how-is-it-detected",level:2}];function l(e){const n={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,a.R)(),...e.components};return(0,o.jsxs)(o.Fragment,{children:[(0,o.jsx)(n.header,{children:(0,o.jsx)(n.h1,{id:"avoid-panic-error",children:"Avoid panic error"})}),"\n",(0,o.jsx)(n.h2,{id:"description",children:"Description"}),"\n",(0,o.jsxs)(n.ul,{children:["\n",(0,o.jsxs)(n.li,{children:["Category: ",(0,o.jsx)(n.code,{children:"Validations and error handling"})]}),"\n",(0,o.jsxs)(n.li,{children:["Severity: ",(0,o.jsx)(n.code,{children:"Enhancement"})]}),"\n",(0,o.jsxs)(n.li,{children:["Detector: ",(0,o.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-panic-error",children:(0,o.jsx)(n.code,{children:"avoid-panic-error"})})]}),"\n",(0,o.jsxs)(n.li,{children:["Test Cases: ",(0,o.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-panic-error/avoid-panic-error-1",children:(0,o.jsx)(n.code,{children:"avoid-panic-error-1"})})]}),"\n"]}),"\n",(0,o.jsx)(n.p,{children:"The panic! macro is used to stop execution when a condition is not met. This is useful for testing and prototyping, but should be avoided in production code."}),"\n",(0,o.jsxs)(n.p,{children:["Using ",(0,o.jsx)(n.code,{children:"Result"})," as return type for functions that can fail is the idiomatic way to handle errors in Rust. The ",(0,o.jsx)(n.code,{children:"Result"})," type is an enum that can be either ",(0,o.jsx)(n.code,{children:"Ok"})," or ",(0,o.jsx)(n.code,{children:"Err"}),". The ",(0,o.jsx)(n.code,{children:"Err"})," variant can contain an error message. The ",(0,o.jsx)(n.code,{children:"?"})," operator can be used to propagate the error message to the caller."]}),"\n",(0,o.jsx)(n.p,{children:"This way, the caller can decide how to handle the error, although the state of the contract is always reverted on the callee."}),"\n",(0,o.jsx)(n.h2,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,o.jsxs)(n.p,{children:["The usage of ",(0,o.jsx)(n.code,{children:"panic!"})," is not recommended because it will stop the execution of the caller contract. This could lead the contract to an inconsistent state if the execution stops in the middle of state changes. Additionally, if execution stops, it could cause a transaction to fail."]}),"\n",(0,o.jsx)(n.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,o.jsxs)(n.p,{children:["In the following example, the ",(0,o.jsx)(n.code,{children:"panic!"})," command is being used to handle errors, disallowing the caller to handle the error in a different way, and completely stopping execution of the caller contract."]}),"\n",(0,o.jsxs)(n.p,{children:["Consider the following ",(0,o.jsx)(n.code,{children:"Soroban"})," contract:"]}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-rust",children:'pub fn add(env: Env, value: u32) -> u32 {\n        let storage = env.storage().instance();\n        let mut count: u32 = storage.get(&COUNTER).unwrap_or(0);\n        match count.checked_add(value) {\n            Some(value) => count = value,\n            None => panic!("Overflow error"),\n        }\n        storage.set(&COUNTER, &count);\n        storage.extend_ttl(100, 100);\n        count\n    }\n'})}),"\n",(0,o.jsx)(n.p,{children:"The add function takes a value as an argument and adds it to the value stored in the contract's storage. The function first checks if the addition will cause an overflow. If the addition will cause an overflow, the function will panic. If the addition will not cause an overflow, the function will add the value to the contract's storage."}),"\n",(0,o.jsx)(n.p,{children:"The usage of panic! in this example, is not recommended because it will stop the execution of the caller contract. If the method was called by the user, then he will receive ContractTrapped as the only error message."}),"\n",(0,o.jsxs)(n.p,{children:["The code example can be found ",(0,o.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-panic-error/avoid-panic-error-1/vulnerable-example",children:"here"}),"."]}),"\n",(0,o.jsx)(n.h2,{id:"remediated-example",children:"Remediated example"}),"\n",(0,o.jsx)(n.p,{children:"A possible remediation goes as follows:"}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-rust",children:" pub fn add(env: Env, value: u32) -> Result<u32, Error> {\n    let storage = env.storage().instance();\n    let mut count: u32 = storage.get(&COUNTER).unwrap_or(0);\n    match count.checked_add(value) {\n        Some(value) => count = value,\n        None => return Err(Error::OverflowError),\n    }\n    storage.set(&COUNTER, &count);\n    storage.extend_ttl(100, 100);\n    Ok(count)\n}\n\n"})}),"\n",(0,o.jsx)(n.p,{children:"And adding the following Error enum:"}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-rust",children:"#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]\n#[repr(u32)]\npub enum Error {\n    OverflowError = 1,\n}\n"})}),"\n",(0,o.jsxs)(n.p,{children:["By first defining the Error enum and then returning a ",(0,o.jsx)(n.code,{children:"Result<(), Error>"}),", more information is added to the caller and, e.g. the caller contract could decide to revert the transaction or to continue execution."]}),"\n",(0,o.jsxs)(n.p,{children:["The remediated code example can be found ",(0,o.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-panic-error/avoid-panic-error-1/remediated-example",children:"here"}),"."]}),"\n",(0,o.jsx)(n.h2,{id:"how-is-it-detected",children:"How is it detected?"}),"\n",(0,o.jsxs)(n.p,{children:["Checks the use of the macro ",(0,o.jsx)(n.code,{children:"panic!"}),"."]})]})}function h(e={}){const{wrapper:n}={...(0,a.R)(),...e.components};return n?(0,o.jsx)(n,{...e,children:(0,o.jsx)(l,{...e})}):l(e)}}}]);