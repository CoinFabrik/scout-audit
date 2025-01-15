"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[6963],{6421:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>c,contentTitle:()=>o,default:()=>p,frontMatter:()=>r,metadata:()=>s,toc:()=>d});const s=JSON.parse('{"id":"detectors/soroban/unsafe-map-get","title":"Unsafe map get","description":"Description","source":"@site/docs/detectors/soroban/19-unsafe-map-get.md","sourceDirName":"detectors/soroban","slug":"/detectors/soroban/unsafe-map-get","permalink":"/docs/detectors/soroban/unsafe-map-get","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/detectors/soroban/19-unsafe-map-get.md","tags":[],"version":"current","sidebarPosition":19,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Unrestricted transfer from","permalink":"/docs/detectors/soroban/unrestricted-transfer-from"},"next":{"title":"Incorrect exponentiation","permalink":"/docs/detectors/soroban/incorrect-exponentiation"}}');var a=n(5105),i=n(6755);const r={},o="Unsafe map get",c={},d=[{value:"Description",id:"description",level:2},{value:"Why is it bad?",id:"why-is-it-bad",level:2},{value:"Issue example",id:"issue-example",level:2},{value:"Remediated example",id:"remediated-example",level:2},{value:"How is it detected?",id:"how-is-it-detected",level:2}];function l(e){const t={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,i.R)(),...e.components};return(0,a.jsxs)(a.Fragment,{children:[(0,a.jsx)(t.header,{children:(0,a.jsx)(t.h1,{id:"unsafe-map-get",children:"Unsafe map get"})}),"\n",(0,a.jsx)(t.h2,{id:"description",children:"Description"}),"\n",(0,a.jsxs)(t.ul,{children:["\n",(0,a.jsxs)(t.li,{children:["Category: ",(0,a.jsx)(t.code,{children:"Validations and error handling"})]}),"\n",(0,a.jsxs)(t.li,{children:["Severity: ",(0,a.jsx)(t.code,{children:"Medium"})]}),"\n",(0,a.jsxs)(t.li,{children:["Detectors: ",(0,a.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unsafe-map-get",children:(0,a.jsx)(t.code,{children:"unsafe-map-get"})})]}),"\n",(0,a.jsxs)(t.li,{children:["Test Cases: ",(0,a.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-map-get/unsafe-map-get-1",children:(0,a.jsx)(t.code,{children:"unsafe-map-get-1"})})]}),"\n"]}),"\n",(0,a.jsxs)(t.p,{children:["The use of certain methods (",(0,a.jsx)(t.code,{children:"get"}),", ",(0,a.jsx)(t.code,{children:"get_unchecked"}),", ",(0,a.jsx)(t.code,{children:"try_get_unchecked"}),") on a ",(0,a.jsx)(t.code,{children:"Map"})," object in the Soroban environment without appropriate error handling can lead to potential runtime panics. This issue stems from accessing the map's values with keys that may not exist, without using safer alternatives that check the existence of the key."]}),"\n",(0,a.jsx)(t.h2,{id:"why-is-it-bad",children:"Why is it bad?"}),"\n",(0,a.jsx)(t.p,{children:"These methods can lead to panics if the key does not exist in the map. Using these methods without proper checks increases the risk of runtime errors that can disrupt the execution of the smart contract and potentially lead to unexpected behavior or denial of service."}),"\n",(0,a.jsx)(t.h2,{id:"issue-example",children:"Issue example"}),"\n",(0,a.jsxs)(t.p,{children:["Consider the following ",(0,a.jsx)(t.code,{children:"Soroban"})," contract:"]}),"\n",(0,a.jsx)(t.pre,{children:(0,a.jsx)(t.code,{className:"language-rust",children:"pub fn get_from_map(env: Env) -> Option<i32> {\n    let map: Map<Val, Val> = map![&env, (1i32.into_val(&env), 2i64.into_val(&env))];\n    let map: Val = map.into();\n    let map: Map<i32, i32> = map.try_into_val(&env).unwrap();\n    map.get(1)\n}\n"})}),"\n",(0,a.jsxs)(t.p,{children:["The code example can be found ",(0,a.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-map-get/unsafe-map-get-1/vulnerable-example",children:"here"}),"."]}),"\n",(0,a.jsx)(t.h2,{id:"remediated-example",children:"Remediated example"}),"\n",(0,a.jsx)(t.pre,{children:(0,a.jsx)(t.code,{className:"language-rust",children:"pub fn get_map_with_different_values(env: Env, key: i32) -> Result<Option<i32>, Error> {\n    let map: Map<Val, Val> = map![\n        &env,\n        (1i32.into_val(&env), 2i32.into_val(&env)),\n        (3i32.into_val(&env), 4i64.into_val(&env)),\n    ];\n    let map: Val = map.into();\n    let map: Map<i32, i32> = map.try_into_val(&env).unwrap();\n    map.try_get(key).map_err(Error::from)\n}\n"})}),"\n",(0,a.jsxs)(t.p,{children:["The remediated code example can be found ",(0,a.jsx)(t.a,{href:"https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-map-get/unsafe-map-get-1/remediated-example",children:"here"}),"."]}),"\n",(0,a.jsx)(t.h2,{id:"how-is-it-detected",children:"How is it detected?"}),"\n",(0,a.jsx)(t.p,{children:"Checks for array pushes without access control."})]})}function p(e={}){const{wrapper:t}={...(0,i.R)(),...e.components};return t?(0,a.jsx)(t,{...e,children:(0,a.jsx)(l,{...e})}):l(e)}},6755:(e,t,n)=>{n.d(t,{R:()=>r,x:()=>o});var s=n(8101);const a={},i=s.createContext(a);function r(e){const t=s.useContext(i);return s.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function o(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(a):e.components||a:r(e.components),s.createElement(i.Provider,{value:t},e.children)}}}]);