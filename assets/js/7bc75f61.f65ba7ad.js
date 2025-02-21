"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[1544],{3881:(e,t,n)=>{n.d(t,{R:()=>i,x:()=>a});var s=n(8101);const c={},r=s.createContext(c);function i(e){const t=s.useContext(r);return s.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function a(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(c):e.components||c:i(e.components),s.createElement(r.Provider,{value:t},e.children)}},5222:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>o,contentTitle:()=>a,default:()=>h,frontMatter:()=>i,metadata:()=>s,toc:()=>d});const s=JSON.parse('{"id":"detectors/rust/unsafe-expect","title":"Unsafe expect","description":"Description","source":"@site/docs/detectors/rust/unsafe-expect.md","sourceDirName":"detectors/rust","slug":"/detectors/rust/unsafe-expect","permalink":"/scout-audit/docs/detectors/rust/unsafe-expect","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/rust/unsafe-expect.md","tags":[],"version":"current","frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Overflow-check","permalink":"/scout-audit/docs/detectors/rust/overflow-check"},"next":{"title":"Unsafe unwrap","permalink":"/scout-audit/docs/detectors/rust/unsafe-unwrap"}}');var c=n(5105),r=n(3881);const i={},a="Unsafe expect",o={},d=[{value:"Description",id:"description",level:2},{value:"Why is this bad?",id:"why-is-this-bad",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediated example",id:"remediated-example",level:2},{value:"How is it detected?",id:"how-is-it-detected",level:2}];function l(e){const t={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,r.R)(),...e.components};return(0,c.jsxs)(c.Fragment,{children:[(0,c.jsx)(t.header,{children:(0,c.jsx)(t.h1,{id:"unsafe-expect",children:"Unsafe expect"})}),"\n",(0,c.jsx)(t.h2,{id:"description",children:"Description"}),"\n",(0,c.jsxs)(t.ul,{children:["\n",(0,c.jsxs)(t.li,{children:["Category: ",(0,c.jsx)(t.code,{children:"Validations and error handling"})]}),"\n",(0,c.jsxs)(t.li,{children:["Severity: ",(0,c.jsx)(t.code,{children:"Minor"})]}),"\n",(0,c.jsxs)(t.li,{children:["Detectors: ",(0,c.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-audit/blob/main/detectors/substrate-pallets/unsafe-expect/src/lib.rs",children:(0,c.jsx)(t.code,{children:"unsafe-expect"})})]}),"\n"]}),"\n",(0,c.jsxs)(t.p,{children:["In Rust, the ",(0,c.jsx)(t.code,{children:"expect"})," method is often used for error handling. It returns the contained ",(0,c.jsx)(t.code,{children:"Ok"})," value for a ",(0,c.jsx)(t.code,{children:"Result"})," or ",(0,c.jsx)(t.code,{children:"Some"})," value for an ",(0,c.jsx)(t.code,{children:"Option"}),". If an error occurs, it calls ",(0,c.jsx)(t.code,{children:"panic!"})," with a provided error message."]}),"\n",(0,c.jsx)(t.h2,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,c.jsxs)(t.p,{children:[(0,c.jsx)(t.code,{children:".expect()"})," might panic if the result value is an error or ",(0,c.jsx)(t.code,{children:"None"}),". It is recommended to avoid the panic of a contract because it stops its execution, which might lead the contract to an inconsistent state if the panic occurs in the middle of state changes. Additionally, the panic could cause a transaction to fail."]}),"\n",(0,c.jsx)(t.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,c.jsx)(t.p,{children:"Consider the following snippet code:"}),"\n",(0,c.jsx)(t.pre,{children:(0,c.jsx)(t.code,{className:"language-rust",children:'pub fn balance_of(env: Env, owner: Address) -> i128 {\n    let state = Self::get_state(env);\n    state.balances.get(owner).expect("could not get balance")\n}\n'})}),"\n",(0,c.jsxs)(t.p,{children:["In this contract, the ",(0,c.jsx)(t.code,{children:"balance_of"})," function uses the expect method to retrieve the balance of an account. If there is no entry for this account in the balances mapping, the contract will panic and halt execution, which could be exploited maliciously to disrupt the contract's operation."]}),"\n",(0,c.jsx)(t.h2,{id:"remediated-example",children:"Remediated example"}),"\n",(0,c.jsxs)(t.p,{children:["Instead of using ",(0,c.jsx)(t.code,{children:"expect"}),", use a safer method for error handling. In this case, if there is no entry for an account in the ",(0,c.jsx)(t.code,{children:"balances"})," mapping, return a default value (like ",(0,c.jsx)(t.code,{children:"0"}),")."]}),"\n",(0,c.jsx)(t.pre,{children:(0,c.jsx)(t.code,{className:"language-rust",children:"pub fn balance_of(env: Env, owner: Address) -> i128 {\n    let state = Self::get_state(env);\n    state.balances.get(owner).unwrap_or(0)\n}\n"})}),"\n",(0,c.jsx)(t.h2,{id:"how-is-it-detected",children:"How is it detected?"}),"\n",(0,c.jsxs)(t.p,{children:["Checks for usage of ",(0,c.jsx)(t.code,{children:".expect()"}),"."]})]})}function h(e={}){const{wrapper:t}={...(0,r.R)(),...e.components};return t?(0,c.jsx)(t,{...e,children:(0,c.jsx)(l,{...e})}):l(e)}}}]);