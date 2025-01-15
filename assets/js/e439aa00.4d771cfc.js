"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[902],{2939:(e,t,s)=>{s.r(t),s.d(t,{assets:()=>c,contentTitle:()=>o,default:()=>p,frontMatter:()=>a,metadata:()=>n,toc:()=>l});const n=JSON.parse('{"id":"detectors/substrate/empty-expect","title":"Empty expect","description":"Description","source":"@site/docs/detectors/substrate/empty-expect.md","sourceDirName":"detectors/substrate","slug":"/detectors/substrate/empty-expect","permalink":"/docs/detectors/substrate/empty-expect","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/substrate/empty-expect.md","tags":[],"version":"current","frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Avoid DispatchError::Other()","permalink":"/docs/detectors/substrate/avoid-dispatch-error-other"},"next":{"title":"Equal addresses","permalink":"/docs/detectors/substrate/equal-addresses"}}');var i=s(5105),r=s(6755);const a={},o="Empty expect",c={},l=[{value:"Description",id:"description",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediation",id:"remediation",level:2}];function d(e){const t={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,r.R)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(t.header,{children:(0,i.jsx)(t.h1,{id:"empty-expect",children:"Empty expect"})}),"\n",(0,i.jsx)(t.h2,{id:"description",children:"Description"}),"\n",(0,i.jsxs)(t.ul,{children:["\n",(0,i.jsxs)(t.li,{children:["Category: ",(0,i.jsx)(t.code,{children:"Best Practices"})]}),"\n",(0,i.jsxs)(t.li,{children:["Severity: ",(0,i.jsx)(t.code,{children:"Medium"})]}),"\n",(0,i.jsxs)(t.li,{children:["Detectors: ",(0,i.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/develop/detectors/rust/empty-expect",children:(0,i.jsx)(t.code,{children:"empty-expect"})})]}),"\n",(0,i.jsxs)(t.li,{children:["Test Cases: ",(0,i.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/empty-expect",children:(0,i.jsx)(t.code,{children:"empty-expect-1"})})]}),"\n"]}),"\n",(0,i.jsxs)(t.p,{children:["An empty ",(0,i.jsx)(t.code,{children:".expect()"})," creates a panic without any explanatory message, leaving developers without information to diagnose the error or trace its origin. This lack of clarity can lead to longer resolution times, poor maintenance practices, and potentially even security issues if sensitive operations fail without explanation."]}),"\n",(0,i.jsx)(t.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,i.jsx)(t.p,{children:"Consider the following function:"}),"\n",(0,i.jsx)(t.pre,{children:(0,i.jsx)(t.code,{className:"language-rust",children:'#[pallet::call_index(0)]\npub fn unsafe_get_storage(origin: OriginFor<T>) -> DispatchResult {\n    let who = ensure_signed(origin)?;\n    let example_storage = ExampleStorage::<T>::get();\n    if example_storage.is_some() {\n        let value = example_storage.expect("");\n        Self::deposit_event(Event::UnsafeGetStorage { who, value });\n    }\n    Ok(())\n}\n'})}),"\n",(0,i.jsxs)(t.p,{children:["In the the ",(0,i.jsx)(t.code,{children:"unsafe_get_storage"})," function, the line ",(0,i.jsx)(t.code,{children:'example_storage.expect("")'})," uses an empty string in the ",(0,i.jsx)(t.code,{children:".expect()"})," method. This is problematic because it provides no context for the panic that occurs if the ",(0,i.jsx)(t.code,{children:"Option"})," is ",(0,i.jsx)(t.code,{children:"None"}),". If a panic is triggered, debugging the issue becomes significantly harder, as there is no information to explain what went wrong or why the code expected a value in the storage."]}),"\n",(0,i.jsxs)(t.p,{children:["The vulnerable code example can be found ",(0,i.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/empty-expect/vulnerable/vulnerable-1",children:"here"}),"."]}),"\n",(0,i.jsx)(t.h2,{id:"remediation",children:"Remediation"}),"\n",(0,i.jsxs)(t.p,{children:["Make the ",(0,i.jsx)(t.code,{children:".expect()"})," method include a descriptive message. This change ensures that if the ",(0,i.jsx)(t.code,{children:"Option"})," is ",(0,i.jsx)(t.code,{children:"None"})," and a panic occurs, the message clearly explains the problem."]}),"\n",(0,i.jsx)(t.pre,{children:(0,i.jsx)(t.code,{className:"language-rust",children:'#[pallet::call_index(0)]\npub fn unsafe_get_storage(origin: OriginFor<T>) -> DispatchResult {\n    let who = ensure_signed(origin)?;\n    let example_storage = ExampleStorage::<T>::get();\n    if example_storage.is_some() {\n        let value = example_storage.expect("Storage is not initialized");\n        Self::deposit_event(Event::UnsafeGetStorage { who, value });\n    }\n    Ok(())\n}\n'})}),"\n",(0,i.jsxs)(t.p,{children:["The remediated code example can be found ",(0,i.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/empty-expect/remediated/remediated-1",children:"here"}),"."]})]})}function p(e={}){const{wrapper:t}={...(0,r.R)(),...e.components};return t?(0,i.jsx)(t,{...e,children:(0,i.jsx)(d,{...e})}):d(e)}},6755:(e,t,s)=>{s.d(t,{R:()=>a,x:()=>o});var n=s(8101);const i={},r=n.createContext(i);function a(e){const t=n.useContext(r);return n.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function o(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(i):e.components||i:a(e.components),n.createElement(r.Provider,{value:t},e.children)}}}]);