"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[2952],{6536:(e,s,t)=>{t.r(s),t.d(s,{assets:()=>c,contentTitle:()=>r,default:()=>u,frontMatter:()=>a,metadata:()=>n,toc:()=>d});const n=JSON.parse('{"id":"detectors/rust/avoid-unsafe-block","title":"Avoid unsafe block","description":"Description","source":"@site/docs/detectors/rust/avoid-unsafe-block.md","sourceDirName":"detectors/rust","slug":"/detectors/rust/avoid-unsafe-block","permalink":"/scout-audit/docs/detectors/rust/avoid-unsafe-block","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/rust/avoid-unsafe-block.md","tags":[],"version":"current","frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Avoid panic error","permalink":"/scout-audit/docs/detectors/rust/avoid-panic-error"},"next":{"title":"Divide before multiply","permalink":"/scout-audit/docs/detectors/rust/divide-before-multiply"}}');var i=t(5105),o=t(6755);const a={},r="Avoid unsafe block",c={},d=[{value:"Description",id:"description",level:2},{value:"Why is this bad?",id:"why-is-this-bad",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediated example",id:"remediated-example",level:2},{value:"How is it detected?",id:"how-is-it-detected",level:2}];function l(e){const s={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,o.R)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(s.header,{children:(0,i.jsx)(s.h1,{id:"avoid-unsafe-block",children:"Avoid unsafe block"})}),"\n",(0,i.jsx)(s.h2,{id:"description",children:"Description"}),"\n",(0,i.jsxs)(s.ul,{children:["\n",(0,i.jsxs)(s.li,{children:["Category: ",(0,i.jsx)(s.code,{children:"Validations and error handling"})]}),"\n",(0,i.jsxs)(s.li,{children:["Severity: ",(0,i.jsx)(s.code,{children:"Critical"})]}),"\n",(0,i.jsxs)(s.li,{children:["Detector: ",(0,i.jsx)(s.a,{href:"https://github.com/CoinFabrik/scout-audit/blob/main/detectors/rust/avoid-unsafe-block/src/lib.rs",children:(0,i.jsx)(s.code,{children:"avoid-unsafe-block"})})]}),"\n"]}),"\n",(0,i.jsx)(s.p,{children:"The use of unsafe blocks in Rust is generally discouraged due to the potential risks it poses to the safety and reliability of the code. Rust's primary appeal lies in its ability to provide memory safety guarantees, which are largely enforced through its ownership and type systems. When you enter an unsafe block, you're effectively bypassing these safety checks. These blocks require the programmer to manually ensure that memory is correctly managed and accessed, which is prone to human error and can be challenging even for experienced developers. Therefore, unsafe blocks should only be used when absolutely necessary and when the safety of the operations within can be assured."}),"\n",(0,i.jsx)(s.h2,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,i.jsxs)(s.p,{children:[(0,i.jsx)(s.code,{children:"unsafe"})," blocks should not be used unless absolutely necessary. The use of unsafe blocks in Rust is discouraged because they bypass Rust's memory safety checks, potentially leading to issues like undefined behavior and security vulnerabilities."]}),"\n",(0,i.jsx)(s.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,i.jsxs)(s.p,{children:["Consider the following ",(0,i.jsx)(s.code,{children:"Soroban"})," contract:"]}),"\n",(0,i.jsx)(s.pre,{children:(0,i.jsx)(s.code,{className:"language-rust",children:"#[contractimpl]\nimpl AvoidUnsafeBlock {\n    pub fn unsafe_function(n: u64) -> u64 {\n        unsafe {\n            let mut i = n as f64;\n            let mut y = i.to_bits();\n            y = 0x5fe6ec85e7de30da - (y >> 1);\n            i = f64::from_bits(y);\n            i *= 1.5 - 0.5 * n as f64 * i * i;\n            i *= 1.5 - 0.5 * n as f64 * i * i;\n\n            let result_ptr: *mut f64 = &mut i;\n\n            (*result_ptr).to_bits()\n        }\n    }\n}\n"})}),"\n",(0,i.jsxs)(s.p,{children:["In this example we can see that it creates a raw pointer named ",(0,i.jsx)(s.code,{children:"result_ptr"}),". Then ",(0,i.jsx)(s.code,{children:"(*result_ptr).to_bits()"})," dereferences the raw pointer. This directly accesses the memory location and calls the ",(0,i.jsx)(s.code,{children:"to_bits"})," method on the value stored at that location."]}),"\n",(0,i.jsx)(s.p,{children:"Raw pointers bypass Rust's type safety system and memory management features. If something goes wrong with the calculations or the value of n, dereferencing the pointer could lead to a memory access violations or undefined behavior."}),"\n",(0,i.jsxs)(s.p,{children:["The code example can be found ",(0,i.jsx)(s.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-unsafe-block/avoid-unsafe-block-1/vulnerable-example",children:"here"}),"."]}),"\n",(0,i.jsx)(s.h2,{id:"remediated-example",children:"Remediated example"}),"\n",(0,i.jsx)(s.p,{children:"By removing the raw pointer, the following version eliminates the issue associated with dereferencing memory in an unsafe way. Rust's type safety checks ensure memory is accessed correctly, preventing the potential issues mentioned earlier."}),"\n",(0,i.jsx)(s.pre,{children:(0,i.jsx)(s.code,{className:"language-rust",children:" #[contractimpl]\nimpl AvoidUnsafeBlock {\n    pub fn unsafe_function(n: u64) -> u64 {\n        let mut i = n as f64;\n        let mut y = i.to_bits();\n        y = 0x5fe6ec85e7de30da - (y >> 1);\n        i = f64::from_bits(y);\n        i *= 1.5 - 0.5 * n as f64 * i * i;\n        i *= 1.5 - 0.5 * n as f64 * i * i;\n        i.to_bits()\n    }\n}\n"})}),"\n",(0,i.jsxs)(s.p,{children:["The remediated code example can be found ",(0,i.jsx)(s.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-unsafe-block/avoid-unsafe-block-1/remediated-example",children:"here"}),"."]}),"\n",(0,i.jsx)(s.h2,{id:"how-is-it-detected",children:"How is it detected?"}),"\n",(0,i.jsxs)(s.p,{children:["Checks for usage of ",(0,i.jsx)(s.code,{children:"unsafe"})," blocks."]})]})}function u(e={}){const{wrapper:s}={...(0,o.R)(),...e.components};return s?(0,i.jsx)(s,{...e,children:(0,i.jsx)(l,{...e})}):l(e)}},6755:(e,s,t)=>{t.d(s,{R:()=>a,x:()=>r});var n=t(8101);const i={},o=n.createContext(i);function a(e){const s=n.useContext(o);return n.useMemo((function(){return"function"==typeof e?e(s):{...s,...e}}),[s,e])}function r(e){let s;return s=e.disableParentContext?"function"==typeof e.components?e.components(i):e.components||i:a(e.components),n.createElement(o.Provider,{value:s},e.children)}}}]);