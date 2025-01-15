"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[3696],{8950:(e,t,o)=>{o.r(t),o.d(t,{assets:()=>a,contentTitle:()=>c,default:()=>u,frontMatter:()=>l,metadata:()=>n,toc:()=>r});const n=JSON.parse('{"id":"features/toggle-detections","title":"Toggle detections on and off","description":"In addition to enabling and disabling detectors, Scout allows users to toggle individual detections on or off. This feature is useful for disabling detections that are false positives or not relevant to the analyzed codebase.","source":"@site/docs/features/toggle-detections.md","sourceDirName":"features","slug":"/features/toggle-detections","permalink":"/scout-audit/docs/features/toggle-detections","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/features/toggle-detections.md","tags":[],"version":"current","sidebarPosition":50,"frontMatter":{"sidebar_position":50},"sidebar":"tutorialSidebar","previous":{"title":"Scout VS Code Extension","permalink":"/scout-audit/docs/features/vs-code-extension"},"next":{"title":"Learning to Scout Soroban","permalink":"/scout-audit/docs/learning/learning-to-scout-soroban"}}');var s=o(5105),i=o(6755);const l={sidebar_position:50},c="Toggle detections on and off",a={},r=[{value:"Usage",id:"usage",level:2},{value:"1) Import scout-utils package",id:"1-import-scout-utils-package",level:3},{value:"2) Include scout-utils in your Rust file",id:"2-include-scout-utils-in-your-rust-file",level:3},{value:"3) Use scout_allow macro to disable a detection",id:"3-use-scout_allow-macro-to-disable-a-detection",level:3},{value:"Supported scope",id:"supported-scope",level:2},{value:"Unnecesary scout_allow macro detector",id:"unnecesary-scout_allow-macro-detector",level:2}];function d(e){const t={code:"code",h1:"h1",h2:"h2",h3:"h3",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,i.R)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(t.header,{children:(0,s.jsx)(t.h1,{id:"toggle-detections-on-and-off",children:"Toggle detections on and off"})}),"\n",(0,s.jsx)(t.p,{children:"In addition to enabling and disabling detectors, Scout allows users to toggle individual detections on or off. This feature is useful for disabling detections that are false positives or not relevant to the analyzed codebase."}),"\n",(0,s.jsx)(t.h2,{id:"usage",children:"Usage"}),"\n",(0,s.jsx)(t.h3,{id:"1-import-scout-utils-package",children:"1) Import scout-utils package"}),"\n",(0,s.jsxs)(t.p,{children:["To use the toggle detections on/off feature, you\u2019ll need to import the ",(0,s.jsx)(t.code,{children:"scout-utils"})," package into your project, adding the following line to your ",(0,s.jsx)(t.code,{children:"Cargo.toml"}),"."]}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:'scout-utils = "0.1.0"\n'})}),"\n",(0,s.jsx)(t.h3,{id:"2-include-scout-utils-in-your-rust-file",children:"2) Include scout-utils in your Rust file"}),"\n",(0,s.jsx)(t.p,{children:"Include the scout-utils package in the Rust file in which you want to disable detections, adding the following line:"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:"use scout-audit::scout_allow;\n"})}),"\n",(0,s.jsx)(t.h3,{id:"3-use-scout_allow-macro-to-disable-a-detection",children:"3) Use scout_allow macro to disable a detection"}),"\n",(0,s.jsx)(t.p,{children:"To disable a detection, you\u2019ll need to use the scout_allow macro, with the name of the detection to disable as an attribute. For example:"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:"#[scout_allow(unsafe_unwrap)]\n"})}),"\n",(0,s.jsx)(t.p,{children:"Place the macro before the block of code in which you want to disable a detection. For example:"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:'#[scout_allow(unsafe_expect)]\npub fn my_func() {\nlet x: Option<&str> = None;\nx.expect("Something went wrong!");\n}\n'})}),"\n",(0,s.jsx)(t.p,{children:"The macro supports including more than one attribute to disable multiple detections at once. For example:"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-rust",children:"#[scout_allow(unsafe_unwrap, integer_overflow_or_underflow)]\n"})}),"\n",(0,s.jsx)(t.h2,{id:"supported-scope",children:"Supported scope"}),"\n",(0,s.jsxs)(t.p,{children:[(0,s.jsx)(t.code,{children:"scout_allow"})," macro supports disabling detections for the following scopes:"]}),"\n",(0,s.jsxs)(t.ul,{children:["\n",(0,s.jsx)(t.li,{children:"Functions (entire body)"}),"\n",(0,s.jsx)(t.li,{children:"Modules"}),"\n",(0,s.jsx)(t.li,{children:"Structs"}),"\n",(0,s.jsx)(t.li,{children:"Enums"}),"\n",(0,s.jsx)(t.li,{children:"Traits"}),"\n",(0,s.jsx)(t.li,{children:"Impl blocks"}),"\n"]}),"\n",(0,s.jsx)(t.h2,{id:"unnecesary-scout_allow-macro-detector",children:"Unnecesary scout_allow macro detector"}),"\n",(0,s.jsx)(t.p,{children:"If Scout Audit detects a scout_allow macro for a block of code in which the disallowed detection is not triggered, it will raise a warning."})]})}function u(e={}){const{wrapper:t}={...(0,i.R)(),...e.components};return t?(0,s.jsx)(t,{...e,children:(0,s.jsx)(d,{...e})}):d(e)}},6755:(e,t,o)=>{o.d(t,{R:()=>l,x:()=>c});var n=o(8101);const s={},i=n.createContext(s);function l(e){const t=n.useContext(i);return n.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function c(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:l(e.components),n.createElement(i.Provider,{value:t},e.children)}}}]);