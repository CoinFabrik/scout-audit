"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[8653],{6464:(e,r,t)=>{t.r(r),t.d(r,{assets:()=>d,contentTitle:()=>o,default:()=>u,frontMatter:()=>a,metadata:()=>n,toc:()=>c});const n=JSON.parse('{"id":"detectors/substrate/avoid-dispatch-error-other","title":"Avoid DispatchError::Other()","description":"Description","source":"@site/docs/detectors/substrate/avoid-dispatch-error-other.md","sourceDirName":"detectors/substrate","slug":"/detectors/substrate/avoid-dispatch-error-other","permalink":"/docs/detectors/substrate/avoid-dispatch-error-other","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/substrate/avoid-dispatch-error-other.md","tags":[],"version":"current","frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Avoid debug!() info!()","permalink":"/docs/detectors/substrate/avoid-debug-info"},"next":{"title":"Empty expect","permalink":"/docs/detectors/substrate/empty-expect"}}');var i=t(5105),s=t(6755);const a={},o="Avoid DispatchError::Other()",d={},c=[{value:"Description",id:"description",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediation",id:"remediation",level:2}];function l(e){const r={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,s.R)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(r.header,{children:(0,i.jsx)(r.h1,{id:"avoid-dispatcherrorother",children:"Avoid DispatchError::Other()"})}),"\n",(0,i.jsx)(r.h2,{id:"description",children:"Description"}),"\n",(0,i.jsxs)(r.ul,{children:["\n",(0,i.jsxs)(r.li,{children:["Category: ",(0,i.jsx)(r.code,{children:"Error handling"})]}),"\n",(0,i.jsxs)(r.li,{children:["Severity: ",(0,i.jsx)(r.code,{children:"Enhancement"})]}),"\n",(0,i.jsxs)(r.li,{children:["Detectors: ",(0,i.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/main/detectors/substrate-pallets/avoid-dispatcherror-other",children:(0,i.jsx)(r.code,{children:"avoid-dispatch-error-other"})})]}),"\n",(0,i.jsxs)(r.li,{children:["Test Cases: ",(0,i.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/substrate-pallets/avoid-dispatcherror-other",children:(0,i.jsx)(r.code,{children:"avoid-dispatch-error-other-1"})})]}),"\n"]}),"\n",(0,i.jsxs)(r.p,{children:["Using ",(0,i.jsx)(r.code,{children:"DispatchError::Other()"})," makes error handling challenging, particularly for developers working with smart contracts. The indiscriminate use of this error type makes it difficult to monitor and diagnose specific errors, impeding efficient troubleshooting and code improvement."]}),"\n",(0,i.jsx)(r.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,i.jsx)(r.p,{children:"Consider the following function:"}),"\n",(0,i.jsx)(r.pre,{children:(0,i.jsx)(r.code,{className:"language-rust",children:'#[pallet::call_index(0)]\npub fn accumulate_dummy(origin: OriginFor<T>, increase_by: T::Balance) -> DispatchResult {\n    if increase_by > T::Balance::from(1000u32) {\n        return Err(DispatchError::Other("increase_by is too large"));\n    }\n\n    let _sender = ensure_signed(origin)?;\n\n    <Dummy<T>>::mutate(|dummy| {\n        let new_dummy = dummy.map_or(increase_by, |d| d.saturating_add(increase_by));\n        *dummy = Some(new_dummy);\n    });\n\n    Self::deposit_event(Event::AccumulateDummy {\n        balance: increase_by,\n    });\n\n    Ok(())\n}\n'})}),"\n",(0,i.jsxs)(r.p,{children:["In this code, using ",(0,i.jsx)(r.code,{children:'DispatchError::Other("increase_by is too large")'})," creates a vague error message that does not clearly identify the problem. This generic error handling approach reduces the ability to effectively monitor and debug the code, hindering developers from quickly identifying and resolving the issue."]}),"\n",(0,i.jsxs)(r.p,{children:["The vulnerable code example can be found ",(0,i.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/substrate-pallets/avoid-dispatcherror-other/vulnerable/vulnerable-1",children:"here"}),"."]}),"\n",(0,i.jsx)(r.h2,{id:"remediation",children:"Remediation"}),"\n",(0,i.jsx)(r.p,{children:"To improve error handling, use a specific error variant defined in your pallet. This way, the error is not only more descriptive but also tied to a well-defined variant, which makes it easier for developers to pinpoint the cause of a failure and address it efficiently."}),"\n",(0,i.jsx)(r.pre,{children:(0,i.jsx)(r.code,{className:"language-rust",children:"#[pallet::call_index(0)]\npub fn accumulate_dummy(origin: OriginFor<T>, increase_by: T::Balance) -> DispatchResult {\n    if increase_by > T::Balance::from(1000u32) {\n        return Err(Error::<T>::IncreaseByTooLarge.into());\n    }\n\n    let _sender = ensure_signed(origin)?;\n\n    <Dummy<T>>::mutate(|dummy| {\n        let new_dummy = dummy.map_or(increase_by, |d| d.saturating_add(increase_by));\n        *dummy = Some(new_dummy);\n    });\n\n    Self::deposit_event(Event::AccumulateDummy {\n        balance: increase_by,\n    });\n\n    Ok(())\n}\n"})}),"\n",(0,i.jsxs)(r.p,{children:["The remediated code example can be found ",(0,i.jsx)(r.a,{href:"https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/substrate-pallets/avoid-dispatcherror-other/remediated/remediated-1",children:"here"}),"."]})]})}function u(e={}){const{wrapper:r}={...(0,s.R)(),...e.components};return r?(0,i.jsx)(r,{...e,children:(0,i.jsx)(l,{...e})}):l(e)}},6755:(e,r,t)=>{t.d(r,{R:()=>a,x:()=>o});var n=t(8101);const i={},s=n.createContext(i);function a(e){const r=n.useContext(s);return n.useMemo((function(){return"function"==typeof e?e(r):{...r,...e}}),[r,e])}function o(e){let r;return r=e.disableParentContext?"function"==typeof e.components?e.components(i):e.components||i:a(e.components),n.createElement(s.Provider,{value:r},e.children)}}}]);