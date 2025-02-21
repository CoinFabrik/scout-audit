"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[958],{3881:(e,n,t)=>{t.d(n,{R:()=>s,x:()=>a});var o=t(8101);const i={},r=o.createContext(i);function s(e){const n=o.useContext(r);return o.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function a(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(i):e.components||i:s(e.components),o.createElement(r.Provider,{value:n},e.children)}},5447:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>c,contentTitle:()=>a,default:()=>h,frontMatter:()=>s,metadata:()=>o,toc:()=>d});const o=JSON.parse('{"id":"detectors/soroban/incorrect-exponentiation","title":"Incorrect exponentiation","description":"Description","source":"@site/docs/detectors/soroban/20-incorrect-exponentiation.md","sourceDirName":"detectors/soroban","slug":"/detectors/soroban/incorrect-exponentiation","permalink":"/scout-audit/docs/detectors/soroban/incorrect-exponentiation","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/soroban/20-incorrect-exponentiation.md","tags":[],"version":"current","sidebarPosition":20,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Unsafe map get","permalink":"/scout-audit/docs/detectors/soroban/unsafe-map-get"},"next":{"title":"Integer overflow or underflow","permalink":"/scout-audit/docs/detectors/soroban/integer-overflow-or-underflow"}}');var i=t(5105),r=t(3881);const s={},a="Incorrect exponentiation",c={},d=[{value:"Description",id:"description",level:2},{value:"Why is it bad?",id:"why-is-it-bad",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediated example",id:"remediated-example",level:2},{value:"How is it detected?",id:"how-is-it-detected",level:2},{value:"References",id:"references",level:2}];function l(e){const n={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,r.R)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(n.header,{children:(0,i.jsx)(n.h1,{id:"incorrect-exponentiation",children:"Incorrect exponentiation"})}),"\n",(0,i.jsx)(n.h2,{id:"description",children:"Description"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:["Issue Category: ",(0,i.jsx)(n.code,{children:"Arithmetic"})]}),"\n",(0,i.jsxs)(n.li,{children:["Issue Severity: ",(0,i.jsx)(n.code,{children:"Critical"})]}),"\n",(0,i.jsxs)(n.li,{children:["Detectors: ",(0,i.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/incorrect-exponentiation",children:(0,i.jsx)(n.code,{children:"incorrect-exponentiation"})})]}),"\n",(0,i.jsxs)(n.li,{children:["Test Cases: ",(0,i.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/incorrect-exponentiation/incorrect-exponentiation-1",children:(0,i.jsx)(n.code,{children:"incorrect-exponentiation-1"})})]}),"\n"]}),"\n",(0,i.jsxs)(n.p,{children:["The operator ",(0,i.jsx)(n.code,{children:"^"})," is not an exponential operator, it is a bitwise XOR. Make sure to use ",(0,i.jsx)(n.code,{children:"pow()"})," instead for exponentiation. In case of performing a XOR operation, use ",(0,i.jsx)(n.code,{children:".bitxor()"})," for clarity."]}),"\n",(0,i.jsx)(n.h2,{id:"why-is-it-bad",children:"Why is it bad?"}),"\n",(0,i.jsx)(n.p,{children:"It can produce unexpected behaviour in the smart contract."}),"\n",(0,i.jsx)(n.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,i.jsxs)(n.p,{children:["In the following example, the ",(0,i.jsx)(n.code,{children:"^"})," operand is being used for exponentiation. But in Rust, ",(0,i.jsx)(n.code,{children:"^"})," is the operand for an XOR operation. If misused, this could lead to unexpected behaviour in our contract."]}),"\n",(0,i.jsxs)(n.p,{children:["Consider the following ",(0,i.jsx)(n.code,{children:"Soroban"})," contract:"]}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-rust",children:'   pub fn exp_data_3(e: Env) -> u128 {\n        let mut data = e.storage()\n        .instance()\n        .get::<DataKey, u128>(&DataKey::Data)\n        .expect("Data not found");\n\n        data ^= 3;\n        data\n    }\n'})}),"\n",(0,i.jsxs)(n.p,{children:["The code example can be found ",(0,i.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/incorrect-exponentiation/incorrect-exponentiation-1/vulnerable-example",children:"here"}),"."]}),"\n",(0,i.jsx)(n.h2,{id:"remediated-example",children:"Remediated example"}),"\n",(0,i.jsxs)(n.p,{children:["A possible solution is to use the method ",(0,i.jsx)(n.code,{children:"pow()"}),". But, if a XOR operation is wanted, ",(0,i.jsx)(n.code,{children:".bitxor()"})," method is recommended."]}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-rust",children:'    pub fn exp_data_3(e: Env) -> u128 {\n        let data = e.storage()\n        .instance()\n        .get::<DataKey, u128>(&DataKey::Data)\n        .expect("Data not found");\n\n        data.pow(3)\n    }\n'})}),"\n",(0,i.jsxs)(n.p,{children:["The remediated code example can be found ",(0,i.jsx)(n.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/incorrect-exponentiation/incorrect-exponentiation-1/remediated-example",children:"here"}),"."]}),"\n",(0,i.jsx)(n.h2,{id:"how-is-it-detected",children:"How is it detected?"}),"\n",(0,i.jsxs)(n.p,{children:["Warns about ",(0,i.jsx)(n.code,{children:"^"})," being a ",(0,i.jsx)(n.code,{children:"bit XOR"})," operation instead of an exponentiation."]}),"\n",(0,i.jsx)(n.h2,{id:"references",children:"References"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:(0,i.jsx)(n.a,{href:"https://doc.rust-lang.org/std/ops/trait.BitXor.html",children:"https://doc.rust-lang.org/std/ops/trait.BitXor.html"})}),"\n"]})]})}function h(e={}){const{wrapper:n}={...(0,r.R)(),...e.components};return n?(0,i.jsx)(n,{...e,children:(0,i.jsx)(l,{...e})}):l(e)}}}]);