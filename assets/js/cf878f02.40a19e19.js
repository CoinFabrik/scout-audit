"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[5887],{8035:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>c,contentTitle:()=>a,default:()=>d,frontMatter:()=>r,metadata:()=>o,toc:()=>u});const o=JSON.parse('{"id":"features/scout-github-action","title":"CI/CD Integration","description":"At CoinFabrik, we understand the importance of ensuring code quality and security in every step of the development process. That\'s why we\'ve developed a GitHub action to integrate Scout into the CI/CD pipeline.","source":"@site/docs/features/scout-github-action.md","sourceDirName":"features","slug":"/features/scout-github-action","permalink":"/docs/features/scout-github-action","draft":false,"unlisted":false,"editUrl":"https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/docs/features/scout-github-action.md","tags":[],"version":"current","sidebarPosition":30,"frontMatter":{"sidebar_position":30},"sidebar":"tutorialSidebar","previous":{"title":"Profile configuration","permalink":"/docs/features/profiles"},"next":{"title":"Scout VS Code Extension","permalink":"/docs/features/vs-code-extension"}}');var s=n(5105),i=n(6755);const r={sidebar_position:30},a="CI/CD Integration",c={},u=[{value:"Quick Start",id:"quick-start",level:2},{value:"Considerations",id:"considerations",level:2},{value:"Output Example",id:"output-example",level:2}];function l(e){const t={code:"code",h1:"h1",h2:"h2",header:"header",img:"img",li:"li",ol:"ol",p:"p",pre:"pre",...(0,i.R)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(t.header,{children:(0,s.jsx)(t.h1,{id:"cicd-integration",children:"CI/CD Integration"})}),"\n",(0,s.jsx)(t.p,{children:"At CoinFabrik, we understand the importance of ensuring code quality and security in every step of the development process. That's why we've developed a GitHub action to integrate Scout into the CI/CD pipeline."}),"\n",(0,s.jsx)(t.p,{children:"Scout is triggered upon every commit pushed to a pull request, automatically running the tool against the targeted smart contracts. This immediate feedback loop allows developers to quickly address any issues before merging the code into the main branch, reducing the risk of introducing bugs or vulnerabilities."}),"\n",(0,s.jsx)(t.h2,{id:"quick-start",children:"Quick Start"}),"\n",(0,s.jsxs)(t.p,{children:["To integrate Scout into your CI/CD pipeline, simply add the following ",(0,s.jsx)(t.code,{children:"scout.yml"})," to the ",(0,s.jsx)(t.code,{children:".github/workflows"})," directory in your repo."]}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-yml",children:'name: scout-workflow\non: [push]\n\njobs:\n  scout-audit:\n    runs-on: ubuntu-latest\n    permissions:\n      pull-requests: write\n      contents: write\n      repository-projects: write\n    steps:\n      - name: checkout\n        uses: actions/checkout@v2\n\n      - name: do scout\n        uses: coinfabrik/scout-actions@v2.4\n        with:\n          target: # Path to the root of your smart contract (e.g. contracts/token/)\n          markdown_output: "true"\n\n      - uses: mshick/add-pr-comment@v2.8.2\n        with:\n          message-path: ${{ github.workspace }}/report.md\n'})}),"\n",(0,s.jsx)(t.h2,{id:"considerations",children:"Considerations"}),"\n",(0,s.jsxs)(t.ol,{children:["\n",(0,s.jsx)(t.li,{children:"Make sure that your smart contract compiles correctly. Scout will not run if any compilation errors exist."}),"\n",(0,s.jsxs)(t.li,{children:["Check that ",(0,s.jsx)(t.code,{children:"target"})," in ",(0,s.jsx)(t.code,{children:"scout.yml"})," is set to the root of the smart contract (where the ",(0,s.jsx)(t.code,{children:"Cargo.toml"})," file is)."]}),"\n",(0,s.jsx)(t.li,{children:"To properly see Scout's results, make sure that you have an open pull request to which you are committing your changes, as Scout's results will be shown as a comment in the PR."}),"\n"]}),"\n",(0,s.jsx)(t.h2,{id:"output-example",children:"Output Example"}),"\n",(0,s.jsx)(t.p,{children:"Scout results are display as a comment in the pull request."}),"\n",(0,s.jsx)(t.p,{children:(0,s.jsx)(t.img,{alt:"Scout Action output example.",src:n(8830).A+"",width:"897",height:"1015"})})]})}function d(e={}){const{wrapper:t}={...(0,i.R)(),...e.components};return t?(0,s.jsx)(t,{...e,children:(0,s.jsx)(l,{...e})}):l(e)}},8830:(e,t,n)=>{n.d(t,{A:()=>o});const o=n.p+"assets/images/github-action-output-2003564ac2fa6385cd6a043f45a75ae0.jpg"},6755:(e,t,n)=>{n.d(t,{R:()=>r,x:()=>a});var o=n(8101);const s={},i=o.createContext(s);function r(e){const t=o.useContext(i);return o.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function a(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:r(e.components),o.createElement(i.Provider,{value:t},e.children)}}}]);