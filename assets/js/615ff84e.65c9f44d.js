"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[6646],{3881:(e,t,n)=>{n.d(t,{R:()=>a,x:()=>c});var s=n(8101);const o={},r=s.createContext(o);function a(e){const t=s.useContext(r);return s.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function c(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(o):e.components||o:a(e.components),s.createElement(r.Provider,{value:t},e.children)}},9442:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>i,contentTitle:()=>c,default:()=>l,frontMatter:()=>a,metadata:()=>s,toc:()=>d});const s=JSON.parse('{"id":"detectors/soroban/storage-change-events","title":"Storage change events","description":"Description","source":"@site/docs/detectors/soroban/22-storage-change-events.md","sourceDirName":"detectors/soroban","slug":"/detectors/soroban/storage-change-events","permalink":"/scout-audit/docs/detectors/soroban/storage-change-events","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/soroban/22-storage-change-events.md","tags":[],"version":"current","sidebarPosition":22,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Integer overflow or underflow","permalink":"/scout-audit/docs/detectors/soroban/integer-overflow-or-underflow"},"next":{"title":"Token interface events","permalink":"/scout-audit/docs/detectors/soroban/token-interface-events"}}');var o=n(5105),r=n(3881);const a={},c="Storage change events",i={},d=[{value:"Description",id:"description",level:2},{value:"Why is this bad?",id:"why-is-this-bad",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediated example",id:"remediated-example",level:2},{value:"How is it detected?",id:"how-is-it-detected",level:2}];function h(e){const t={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,r.R)(),...e.components};return(0,o.jsxs)(o.Fragment,{children:[(0,o.jsx)(t.header,{children:(0,o.jsx)(t.h1,{id:"storage-change-events",children:"Storage change events"})}),"\n",(0,o.jsx)(t.h2,{id:"description",children:"Description"}),"\n",(0,o.jsxs)(t.ul,{children:["\n",(0,o.jsxs)(t.li,{children:["Category: ",(0,o.jsx)(t.code,{children:"Best practices"})]}),"\n",(0,o.jsxs)(t.li,{children:["Severity: ",(0,o.jsx)(t.code,{children:"Minor"})]}),"\n",(0,o.jsxs)(t.li,{children:["Detectors: ",(0,o.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/storage-change-events",children:(0,o.jsx)(t.code,{children:"storage-change-events"})})]}),"\n",(0,o.jsxs)(t.li,{children:["Test Cases: ",(0,o.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/storage-change-events/storage-change-events-1",children:(0,o.jsx)(t.code,{children:"storage-change-events-1"})})]}),"\n"]}),"\n",(0,o.jsx)(t.p,{children:"In Rust, it is very important to control storage, since it contains a large part of the information of a contract. For this reason, it is common to control storage movements through events, in order to record the changes that occur. If there is no control over these changes, it can lead to potential problems in the contract."}),"\n",(0,o.jsx)(t.h2,{id:"why-is-this-bad",children:"Why is this bad?"}),"\n",(0,o.jsx)(t.p,{children:"If there is no control over storage changes, it can lead to security and transparency issues within the contract."}),"\n",(0,o.jsx)(t.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,o.jsxs)(t.p,{children:["Consider the following ",(0,o.jsx)(t.code,{children:"Soroban"})," contract:"]}),"\n",(0,o.jsx)(t.pre,{children:(0,o.jsx)(t.code,{className:"language-rust",children:"\n  fn set_counter(env: Env, counter: CounterState) {\n        env.storage().instance().set(&STATE, &counter);\n    }\n\n"})}),"\n",(0,o.jsxs)(t.p,{children:["In this example, the ",(0,o.jsx)(t.code,{children:"set_counter()"})," function does not emit an event to notify of a change in the storage."]}),"\n",(0,o.jsxs)(t.p,{children:["The code example can be found ",(0,o.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/storage-change-events/storage-change-events-1/vulnerable-example",children:"here"}),"."]}),"\n",(0,o.jsx)(t.h2,{id:"remediated-example",children:"Remediated example"}),"\n",(0,o.jsx)(t.pre,{children:(0,o.jsx)(t.code,{className:"language-rust",children:'    fn set_counter(env: Env, counter: CounterState) {\n        env.storage().instance().set(&STATE, &counter);\n        env.events()\n            .publish((COUNTER, symbol_short!("set")), counter.count);\n    }\n'})}),"\n",(0,o.jsxs)(t.p,{children:["In this example, the ",(0,o.jsx)(t.code,{children:"set_counter()"})," function emits an event to notify of a change in the storage."]}),"\n",(0,o.jsxs)(t.p,{children:["The remediated code example can be found ",(0,o.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/storage-change-events/storage-change-events-1/remediated-example",children:"here"}),"."]}),"\n",(0,o.jsx)(t.h2,{id:"how-is-it-detected",children:"How is it detected?"}),"\n",(0,o.jsx)(t.p,{children:"Checks if the function emits an event in case a change has occurred in the storage."})]})}function l(e={}){const{wrapper:t}={...(0,r.R)(),...e.components};return t?(0,o.jsx)(t,{...e,children:(0,o.jsx)(h,{...e})}):h(e)}}}]);