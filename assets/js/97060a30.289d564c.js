"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[7413],{9775:(e,t,i)=>{i.r(t),i.d(t,{assets:()=>l,contentTitle:()=>c,default:()=>u,frontMatter:()=>a,metadata:()=>n,toc:()=>o});const n=JSON.parse('{"id":"detectors/substrate/invalid-extrinsic-weight","title":"Invalid extrinsic weight","description":"Description","source":"@site/docs/detectors/substrate/invalid-extrinsic-weight.md","sourceDirName":"detectors/substrate","slug":"/detectors/substrate/invalid-extrinsic-weight","permalink":"/scout-audit/docs/detectors/substrate/invalid-extrinsic-weight","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/substrate/invalid-extrinsic-weight.md","tags":[],"version":"current","frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Equal addresses","permalink":"/scout-audit/docs/detectors/substrate/equal-addresses"},"next":{"title":"Known vulnerabilities","permalink":"/scout-audit/docs/detectors/substrate/known-vulnerabilities"}}');var s=i(5105),r=i(6755);const a={},c="Invalid extrinsic weight",l={},o=[{value:"Description",id:"description",level:2},{value:"Issue scenario",id:"issue-scenario",level:2},{value:"Remediation",id:"remediation",level:2}];function d(e){const t={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,r.R)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(t.header,{children:(0,s.jsx)(t.h1,{id:"invalid-extrinsic-weight",children:"Invalid extrinsic weight"})}),"\n",(0,s.jsx)(t.h2,{id:"description",children:"Description"}),"\n",(0,s.jsxs)(t.ul,{children:["\n",(0,s.jsxs)(t.li,{children:["Category: ",(0,s.jsx)(t.code,{children:"Known Bugs"})]}),"\n",(0,s.jsxs)(t.li,{children:["Severity: ",(0,s.jsx)(t.code,{children:"Enhancement"})]}),"\n",(0,s.jsxs)(t.li,{children:["Detectors: ",(0,s.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/develop/detectors/substrate-pallets/invalid-extrinsic-weight",children:(0,s.jsx)(t.code,{children:"invalid-extrinsic-weight"})})]}),"\n",(0,s.jsxs)(t.li,{children:["Test Cases: ",(0,s.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/invalid-extrinsic-weight",children:(0,s.jsx)(t.code,{children:"invalid-extrinsic-weight-1"})})]}),"\n"]}),"\n",(0,s.jsx)(t.p,{children:"The weight attribute is using a weight calculation function that doesn't match the extrinsic name. Each extrinsic must have its own dedicated weight calculation to accurately reflect its resource consumption. Reusing weight calculations from other functions can lead to incorrect resource estimation and potential issues in production."}),"\n",(0,s.jsx)(t.h2,{id:"issue-scenario",children:"Issue scenario"}),"\n",(0,s.jsx)(t.p,{children:"Consider the following functions:"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:"#[pallet::call(weight(<T as Config>::WeightInfo))]\nimpl<T: Config> Pallet<T> {\n    #[pallet::call_index(0)]\n    pub fn dummy_call(_origin: OriginFor<T>) -> DispatchResult {\n        Ok(())\n    }\n\n    #[pallet::call_index(1)]\n    pub fn another_dummy_call(_origin: OriginFor<T>) -> DispatchResult {\n        Ok(())\n    }\n}\n"})}),"\n",(0,s.jsxs)(t.p,{children:["In the provided implementation, ",(0,s.jsx)(t.code,{children:"another_dummy_call"})," reuses the weight calculation function intended for another context. By not having a unique weight definition, this extrinsic introduces vulnerabilities into the system. Specifically, reusing weight functions may result in underestimating or overestimating resource consumption, leaving the network susceptible to Denial-of-Service (DoS) attacks."]}),"\n",(0,s.jsxs)(t.p,{children:["The vulnerable code example can be found ",(0,s.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/invalid-extrinsic-weight/vulnerable/vulnerable-1",children:"here"}),"."]}),"\n",(0,s.jsx)(t.h2,{id:"remediation",children:"Remediation"}),"\n",(0,s.jsx)(t.p,{children:"To prevent this issue, assign a unique and dedicated weight calculation function to each extrinsic as in the following example."}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:"#[pallet::call(weight(<T as Config>::WeightInfo))]\nimpl<T: Config> Pallet<T> {\n    #[pallet::call_index(0)]\n    pub fn dummy_call(_origin: OriginFor<T>) -> DispatchResult {\n        Ok(())\n    }\n\n    #[pallet::call_index(1)]\n    #[pallet::weight(<T as Config>::WeightInfo::dummy_call())]\n    pub fn another_dummy_call(_origin: OriginFor<T>) -> DispatchResult {\n        Ok(())\n    }\n}\n"})}),"\n",(0,s.jsxs)(t.p,{children:["The remediated code example can be found ",(0,s.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/invalid-extrinsic-weight/vulnerable/vulnerable-1",children:"here"}),"."]})]})}function u(e={}){const{wrapper:t}={...(0,r.R)(),...e.components};return t?(0,s.jsx)(t,{...e,children:(0,s.jsx)(d,{...e})}):d(e)}},6755:(e,t,i)=>{i.d(t,{R:()=>a,x:()=>c});var n=i(8101);const s={},r=n.createContext(s);function a(e){const t=n.useContext(r);return n.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function c(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:a(e.components),n.createElement(r.Provider,{value:t},e.children)}}}]);