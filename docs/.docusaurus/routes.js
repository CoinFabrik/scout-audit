import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/scout-audit/__docusaurus/debug',
    component: ComponentCreator('/scout-audit/__docusaurus/debug', 'a27'),
    exact: true
  },
  {
    path: '/scout-audit/__docusaurus/debug/config',
    component: ComponentCreator('/scout-audit/__docusaurus/debug/config', '3af'),
    exact: true
  },
  {
    path: '/scout-audit/__docusaurus/debug/content',
    component: ComponentCreator('/scout-audit/__docusaurus/debug/content', '432'),
    exact: true
  },
  {
    path: '/scout-audit/__docusaurus/debug/globalData',
    component: ComponentCreator('/scout-audit/__docusaurus/debug/globalData', '89f'),
    exact: true
  },
  {
    path: '/scout-audit/__docusaurus/debug/metadata',
    component: ComponentCreator('/scout-audit/__docusaurus/debug/metadata', '94a'),
    exact: true
  },
  {
    path: '/scout-audit/__docusaurus/debug/registry',
    component: ComponentCreator('/scout-audit/__docusaurus/debug/registry', 'c71'),
    exact: true
  },
  {
    path: '/scout-audit/__docusaurus/debug/routes',
    component: ComponentCreator('/scout-audit/__docusaurus/debug/routes', '95a'),
    exact: true
  },
  {
    path: '/scout-audit/markdown-page',
    component: ComponentCreator('/scout-audit/markdown-page', '500'),
    exact: true
  },
  {
    path: '/scout-audit/docs',
    component: ComponentCreator('/scout-audit/docs', 'f92'),
    routes: [
      {
        path: '/scout-audit/docs',
        component: ComponentCreator('/scout-audit/docs', '16e'),
        routes: [
          {
            path: '/scout-audit/docs',
            component: ComponentCreator('/scout-audit/docs', '1e3'),
            routes: [
              {
                path: '/scout-audit/docs/detectors/detectors-intro',
                component: ComponentCreator('/scout-audit/docs/detectors/detectors-intro', '4d5'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/assert-violation',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/assert-violation', '7b8'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/avoid-autokey-upgradable',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/avoid-autokey-upgradable', '116'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/avoid-core-mem-forget',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/avoid-core-mem-forget', 'bcc'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/avoid-format-string',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/avoid-format-string', 'c37'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/avoid-unsafe-block',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/avoid-unsafe-block', 'ac2'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/buffering-unsized-types',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/buffering-unsized-types', 'c77'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/delegate-call',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/delegate-call', '698'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/divide-before-multiply',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/divide-before-multiply', 'b29'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/dont-use-instantiate-contract-v1',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/dont-use-instantiate-contract-v1', 'd84'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/dont-use-invoke-contract-v1',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/dont-use-invoke-contract-v1', 'dfb'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/dos-unbounded-operation',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/dos-unbounded-operation', '115'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/dos-unexpected-revert-with-vector',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/dos-unexpected-revert-with-vector', 'aae'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/incorrect-exponentiation',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/incorrect-exponentiation', '32f'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/ink-version',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/ink-version', '904'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/insufficiently-random-values',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/insufficiently-random-values', '537'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/integer-overflow-or-underflow',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/integer-overflow-or-underflow', 'b1b'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/iterators-over-indexing',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/iterators-over-indexing', '2c6'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/lazy-delegate',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/lazy-delegate', 'b99'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/lazy-values-not-set',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/lazy-values-not-set', '8b0'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/non-payable-transferred-value',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/non-payable-transferred-value', '494'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/panic-error',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/panic-error', '14e'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/reentrancy',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/reentrancy', 'b0f'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/set-contract-storage',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/set-contract-storage', 'a17'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/unprotected-mapping-operation',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/unprotected-mapping-operation', '9ca'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/unprotected-self-destruct',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/unprotected-self-destruct', '58d'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/unprotected-set-code-hash',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/unprotected-set-code-hash', 'ba4'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/unrestricted-transfer-from',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/unrestricted-transfer-from', '4a8'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/unsafe-expect',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/unsafe-expect', 'd11'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/unsafe-unwrap',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/unsafe-unwrap', '45b'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/unused-return-enum',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/unused-return-enum', '27c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/vec-could-be-mapping',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/vec-could-be-mapping', '56d'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/warning-sr25519-verify',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/warning-sr25519-verify', 'f66'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/ink/zero-or-test-address',
                component: ComponentCreator('/scout-audit/docs/detectors/ink/zero-or-test-address', '955'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/assert-violation',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/assert-violation', '5e5'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/avoid-core-mem-forget',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/avoid-core-mem-forget', 'f0d'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/avoid-panic-error',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/avoid-panic-error', '6d8'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/avoid-unsafe-block',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/avoid-unsafe-block', 'd9b'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/divide-before-multiply',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/divide-before-multiply', 'dbd'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/dos-unbounded-operation',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/dos-unbounded-operation', '9bf'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/dos-unexpected-revert-with-vector',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/dos-unexpected-revert-with-vector', 'ec7'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/incorrect-exponentiation',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/incorrect-exponentiation', '320'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/insufficiently-random-values',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/insufficiently-random-values', '467'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/integer-overflow -or-underflow',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/integer-overflow -or-underflow', 'a4a'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/iterators-over-indexing',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/iterators-over-indexing', 'a88'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/overflow-check',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/overflow-check', '0f5'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/set-contract-storage',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/set-contract-storage', '5ab'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/soroban-version',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/soroban-version', 'a0c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/storage-change-events',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/storage-change-events', '76b'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/token-interface-events',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/token-interface-events', '60c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/unprotected-mapping-operation',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/unprotected-mapping-operation', '725'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/unprotected-update-current-contract-wasm',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/unprotected-update-current-contract-wasm', 'c9a'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/unrestricted-transfer-from',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/unrestricted-transfer-from', 'aac'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/unsafe-expect',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/unsafe-expect', 'f78'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/unsafe-map-get',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/unsafe-map-get', '7b1'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/unsafe-unwrap',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/unsafe-unwrap', 'ae1'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/soroban/unused-return-enum',
                component: ComponentCreator('/scout-audit/docs/detectors/soroban/unused-return-enum', 'de3'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/substrate/avoid-debug-info',
                component: ComponentCreator('/scout-audit/docs/detectors/substrate/avoid-debug-info', '263'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/substrate/avoid-dispatch-error-other',
                component: ComponentCreator('/scout-audit/docs/detectors/substrate/avoid-dispatch-error-other', 'a3e'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/substrate/empty-expect',
                component: ComponentCreator('/scout-audit/docs/detectors/substrate/empty-expect', '0a4'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/substrate/equal-addresses',
                component: ComponentCreator('/scout-audit/docs/detectors/substrate/equal-addresses', 'a85'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/substrate/invalid-extrinsic-weight',
                component: ComponentCreator('/scout-audit/docs/detectors/substrate/invalid-extrinsic-weight', '21c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/substrate/known-vulnerabilities',
                component: ComponentCreator('/scout-audit/docs/detectors/substrate/known-vulnerabilities', '0c7'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/detectors/substrate/unsigned-extrinsic',
                component: ComponentCreator('/scout-audit/docs/detectors/substrate/unsigned-extrinsic', 'aa6'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/features/command-line-interface',
                component: ComponentCreator('/scout-audit/docs/features/command-line-interface', 'a20'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/features/profiles',
                component: ComponentCreator('/scout-audit/docs/features/profiles', 'bd9'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/features/scout-github-action',
                component: ComponentCreator('/scout-audit/docs/features/scout-github-action', 'c7b'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/features/toggle-detections',
                component: ComponentCreator('/scout-audit/docs/features/toggle-detections', '9d1'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/features/vs-code-extension',
                component: ComponentCreator('/scout-audit/docs/features/vs-code-extension', '6b5'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/intro',
                component: ComponentCreator('/scout-audit/docs/intro', '1d6'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/learning/learning-to-scout-soroban',
                component: ComponentCreator('/scout-audit/docs/learning/learning-to-scout-soroban', '9cf'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/scout-audit/docs/learning/scout-soroban-examples',
                component: ComponentCreator('/scout-audit/docs/learning/scout-soroban-examples', 'c4a'),
                exact: true,
                sidebar: "tutorialSidebar"
              }
            ]
          }
        ]
      }
    ]
  },
  {
    path: '/scout-audit/',
    component: ComponentCreator('/scout-audit/', 'd1f'),
    exact: true
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];
