"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[2674],{3881:(e,t,n)=>{n.d(t,{R:()=>o,x:()=>a});var r=n(8101);const i={},s=r.createContext(i);function o(e){const t=r.useContext(s);return r.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function a(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(i):e.components||i:o(e.components),r.createElement(s.Provider,{value:t},e.children)}},4677:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>l,contentTitle:()=>a,default:()=>u,frontMatter:()=>o,metadata:()=>r,toc:()=>d});const r=JSON.parse('{"id":"detectors/substrate/integer-overflow-or-underflow","title":"Integer overflow or underflow","description":"Description","source":"@site/docs/detectors/substrate/integer-overflow-or-underflow.md","sourceDirName":"detectors/substrate","slug":"/detectors/substrate/integer-overflow-or-underflow","permalink":"/scout-audit/docs/detectors/substrate/integer-overflow-or-underflow","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/substrate/integer-overflow-or-underflow.md","tags":[],"version":"current","frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Incorrect exponentiation","permalink":"/scout-audit/docs/detectors/substrate/incorrect-exponentiation"},"next":{"title":"Invalid extrinsic weight","permalink":"/scout-audit/docs/detectors/substrate/invalid-extrinsic-weight"}}');var i=n(5105),s=n(3881);const o={},a="Integer overflow or underflow",l={},d=[{value:"Description",id:"description",level:2},{value:"Why is this bad?",id:"why-is-this-bad",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediated example",id:"remediated-example",level:2},{value:"How is it detected?",id:"how-is-it-detected",level:2}];function c(e){const t={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,s.R)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(t.header,{children:(0,i.jsx)(t.h1,{id:"integer-overflow-or-underflow",children:"Integer overflow or underflow"})}),"\n",(0,i.jsx)(t.h2,{id:"description",children:"Description"}),"\n",(0,i.jsxs)(t.ul,{children:["\n",(0,i.jsxs)(t.li,{children:["Category: ",(0,i.jsx)(t.code,{children:"Arithmetic"})]}),"\n",(0,i.jsxs)(t.li,{children:["Severity: ",(0,i.jsx)(t.code,{children:"Critical"})]}),"\n",(0,i.jsxs)(t.li,{children:["Detectors: ",(0,i.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-audit/blob/main/detectors/substrate-pallets/integer-overflow-or-underflow/src/lib.rs",children:(0,i.jsx)(t.code,{children:"integer-overflow-or-underflow"})})]}),"\n"]}),"\n",(0,i.jsx)(t.p,{children:"In Rust, arithmetic operations can result in a value that falls outside the allowed numerical range for a given type. When the result exceeds the maximum value of the range, it's called an overflow, and when it falls below the minimum value of the range, it's called an underflow."}),"\n",(0,i.jsx)(t.h2,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,i.jsx)(t.p,{children:"If there are arithmetic operations with overflow or underflow problems, and if errors are not handled correctly, incorrect results will be generated, bringing potential problems for the contract. Additionally, these types of errors can allow attackers to drain a contract\u2019s funds or manipulate its logic."}),"\n",(0,i.jsx)(t.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,i.jsxs)(t.p,{children:["Consider the following ",(0,i.jsx)(t.code,{children:"Substrate pallet"}),":"]}),"\n",(0,i.jsx)(t.pre,{children:(0,i.jsx)(t.code,{className:"language-rust",children:"#[pallet::call_index(0)]\n        pub fn accumulate_dummy(origin: OriginFor<T>, increase_by: T::Balance) -> DispatchResult {\n            let _sender = ensure_signed(origin)?;\n            <Dummy<T>>::mutate(|dummy| {\n                let new_dummy = dummy.map_or(increase_by, |d| d + increase_by);\n                *dummy = Some(new_dummy);\n            });\n            Self::deposit_event(Event::AccumulateDummy {\n                balance: increase_by,\n            });\n            Ok(())\n        }\n"})}),"\n",(0,i.jsxs)(t.p,{children:["In this example, an operation is performed on two u32 (",(0,i.jsx)(t.code,{children:"d"})," and ",(0,i.jsx)(t.code,{children:"increase_by"}),") values without any safeguards against overflow if it occurs."]}),"\n",(0,i.jsx)(t.h2,{id:"remediated-example",children:"Remediated example"}),"\n",(0,i.jsx)(t.p,{children:"Consider using safe operations to prevent an overflow"}),"\n",(0,i.jsx)(t.pre,{children:(0,i.jsx)(t.code,{className:"language-rust",children:"#[pallet::call_index(0)]\n        pub fn accumulate_dummy(origin: OriginFor<T>, increase_by: T::Balance) -> DispatchResult {\n            let _sender = ensure_signed(origin)?;\n            <Dummy<T>>::mutate(|dummy| {\n                let new_dummy = dummy.map_or(increase_by, |d| d.saturating_add(increase_by));\n                *dummy = Some(new_dummy);\n            });\n            Self::deposit_event(Event::AccumulateDummy {\n                balance: increase_by,\n            });\n            Ok(())\n        }\n"})}),"\n",(0,i.jsxs)(t.p,{children:["In this example, the ",(0,i.jsx)(t.code,{children:"saturating_add"})," method is used to perform the addition. It returns the sum if no overflow occurs; otherwise, it returns ",(0,i.jsx)(t.code,{children:"None"}),", with an OverflowError variant indicating that an overflow error has occurred."]}),"\n",(0,i.jsx)(t.h2,{id:"how-is-it-detected",children:"How is it detected?"}),"\n",(0,i.jsx)(t.p,{children:"Checks if there\u2019s any numerical overflow or underflow."})]})}function u(e={}){const{wrapper:t}={...(0,s.R)(),...e.components};return t?(0,i.jsx)(t,{...e,children:(0,i.jsx)(c,{...e})}):c(e)}}}]);