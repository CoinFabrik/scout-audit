import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/__docusaurus/debug',
    component: ComponentCreator('/__docusaurus/debug', '5ff'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/config',
    component: ComponentCreator('/__docusaurus/debug/config', '5ba'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/content',
    component: ComponentCreator('/__docusaurus/debug/content', 'a2b'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/globalData',
    component: ComponentCreator('/__docusaurus/debug/globalData', 'c3c'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/metadata',
    component: ComponentCreator('/__docusaurus/debug/metadata', '156'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/registry',
    component: ComponentCreator('/__docusaurus/debug/registry', '88c'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/routes',
    component: ComponentCreator('/__docusaurus/debug/routes', '000'),
    exact: true
  },
  {
    path: '/markdown-page',
    component: ComponentCreator('/markdown-page', '3d7'),
    exact: true
  },
  {
    path: '/docs',
    component: ComponentCreator('/docs', '1c8'),
    routes: [
      {
        path: '/docs',
        component: ComponentCreator('/docs', '239'),
        routes: [
          {
            path: '/docs',
            component: ComponentCreator('/docs', '1fd'),
            routes: [
              {
                path: '/docs/detectors/detectors-intro',
                component: ComponentCreator('/docs/detectors/detectors-intro', '275'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/assert-violation',
                component: ComponentCreator('/docs/detectors/ink/assert-violation', '4a2'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/avoid-autokey-upgradable',
                component: ComponentCreator('/docs/detectors/ink/avoid-autokey-upgradable', '2fa'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/avoid-core-mem-forget',
                component: ComponentCreator('/docs/detectors/ink/avoid-core-mem-forget', 'caf'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/avoid-format-string',
                component: ComponentCreator('/docs/detectors/ink/avoid-format-string', '3ee'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/avoid-unsafe-block',
                component: ComponentCreator('/docs/detectors/ink/avoid-unsafe-block', 'c78'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/buffering-unsized-types',
                component: ComponentCreator('/docs/detectors/ink/buffering-unsized-types', 'f71'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/delegate-call',
                component: ComponentCreator('/docs/detectors/ink/delegate-call', '860'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/divide-before-multiply',
                component: ComponentCreator('/docs/detectors/ink/divide-before-multiply', '03f'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/dont-use-instantiate-contract-v1',
                component: ComponentCreator('/docs/detectors/ink/dont-use-instantiate-contract-v1', 'dbf'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/dont-use-invoke-contract-v1',
                component: ComponentCreator('/docs/detectors/ink/dont-use-invoke-contract-v1', 'cb9'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/dos-unbounded-operation',
                component: ComponentCreator('/docs/detectors/ink/dos-unbounded-operation', '885'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/dos-unexpected-revert-with-vector',
                component: ComponentCreator('/docs/detectors/ink/dos-unexpected-revert-with-vector', '637'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/incorrect-exponentiation',
                component: ComponentCreator('/docs/detectors/ink/incorrect-exponentiation', '296'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/ink-version',
                component: ComponentCreator('/docs/detectors/ink/ink-version', '0eb'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/insufficiently-random-values',
                component: ComponentCreator('/docs/detectors/ink/insufficiently-random-values', 'f09'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/integer-overflow-or-underflow',
                component: ComponentCreator('/docs/detectors/ink/integer-overflow-or-underflow', 'e87'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/iterators-over-indexing',
                component: ComponentCreator('/docs/detectors/ink/iterators-over-indexing', 'b62'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/lazy-delegate',
                component: ComponentCreator('/docs/detectors/ink/lazy-delegate', 'e47'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/lazy-values-not-set',
                component: ComponentCreator('/docs/detectors/ink/lazy-values-not-set', '1f9'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/non-payable-transferred-value',
                component: ComponentCreator('/docs/detectors/ink/non-payable-transferred-value', 'fd1'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/panic-error',
                component: ComponentCreator('/docs/detectors/ink/panic-error', '65b'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/reentrancy',
                component: ComponentCreator('/docs/detectors/ink/reentrancy', '9c2'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/set-contract-storage',
                component: ComponentCreator('/docs/detectors/ink/set-contract-storage', 'bca'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/unprotected-mapping-operation',
                component: ComponentCreator('/docs/detectors/ink/unprotected-mapping-operation', '283'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/unprotected-self-destruct',
                component: ComponentCreator('/docs/detectors/ink/unprotected-self-destruct', '10d'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/unprotected-set-code-hash',
                component: ComponentCreator('/docs/detectors/ink/unprotected-set-code-hash', '8ec'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/unrestricted-transfer-from',
                component: ComponentCreator('/docs/detectors/ink/unrestricted-transfer-from', '87c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/unsafe-expect',
                component: ComponentCreator('/docs/detectors/ink/unsafe-expect', '3e7'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/unsafe-unwrap',
                component: ComponentCreator('/docs/detectors/ink/unsafe-unwrap', '7fd'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/unused-return-enum',
                component: ComponentCreator('/docs/detectors/ink/unused-return-enum', '10c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/vec-could-be-mapping',
                component: ComponentCreator('/docs/detectors/ink/vec-could-be-mapping', '6e8'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/warning-sr25519-verify',
                component: ComponentCreator('/docs/detectors/ink/warning-sr25519-verify', '60f'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/ink/zero-or-test-address',
                component: ComponentCreator('/docs/detectors/ink/zero-or-test-address', 'e00'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/assert-violation',
                component: ComponentCreator('/docs/detectors/soroban/assert-violation', '9a3'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/avoid-core-mem-forget',
                component: ComponentCreator('/docs/detectors/soroban/avoid-core-mem-forget', 'e1f'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/avoid-panic-error',
                component: ComponentCreator('/docs/detectors/soroban/avoid-panic-error', '48e'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/avoid-unsafe-block',
                component: ComponentCreator('/docs/detectors/soroban/avoid-unsafe-block', '7e4'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/divide-before-multiply',
                component: ComponentCreator('/docs/detectors/soroban/divide-before-multiply', '058'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/dos-unbounded-operation',
                component: ComponentCreator('/docs/detectors/soroban/dos-unbounded-operation', '6dc'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/dos-unexpected-revert-with-vector',
                component: ComponentCreator('/docs/detectors/soroban/dos-unexpected-revert-with-vector', 'c45'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/incorrect-exponentiation',
                component: ComponentCreator('/docs/detectors/soroban/incorrect-exponentiation', '5fd'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/insufficiently-random-values',
                component: ComponentCreator('/docs/detectors/soroban/insufficiently-random-values', '3a6'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/integer-overflow -or-underflow',
                component: ComponentCreator('/docs/detectors/soroban/integer-overflow -or-underflow', '642'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/iterators-over-indexing',
                component: ComponentCreator('/docs/detectors/soroban/iterators-over-indexing', 'de9'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/overflow-check',
                component: ComponentCreator('/docs/detectors/soroban/overflow-check', 'a4f'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/set-contract-storage',
                component: ComponentCreator('/docs/detectors/soroban/set-contract-storage', 'f72'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/soroban-version',
                component: ComponentCreator('/docs/detectors/soroban/soroban-version', 'ad5'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/storage-change-events',
                component: ComponentCreator('/docs/detectors/soroban/storage-change-events', '3f5'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/token-interface-events',
                component: ComponentCreator('/docs/detectors/soroban/token-interface-events', '3f2'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/unprotected-mapping-operation',
                component: ComponentCreator('/docs/detectors/soroban/unprotected-mapping-operation', '9c7'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/unprotected-update-current-contract-wasm',
                component: ComponentCreator('/docs/detectors/soroban/unprotected-update-current-contract-wasm', '161'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/unrestricted-transfer-from',
                component: ComponentCreator('/docs/detectors/soroban/unrestricted-transfer-from', 'e3a'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/unsafe-expect',
                component: ComponentCreator('/docs/detectors/soroban/unsafe-expect', '4e4'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/unsafe-map-get',
                component: ComponentCreator('/docs/detectors/soroban/unsafe-map-get', '03c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/unsafe-unwrap',
                component: ComponentCreator('/docs/detectors/soroban/unsafe-unwrap', 'f3d'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/soroban/unused-return-enum',
                component: ComponentCreator('/docs/detectors/soroban/unused-return-enum', 'e08'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/substrate/avoid-debug-info',
                component: ComponentCreator('/docs/detectors/substrate/avoid-debug-info', '5d5'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/substrate/avoid-dispatch-error-other',
                component: ComponentCreator('/docs/detectors/substrate/avoid-dispatch-error-other', 'bbe'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/substrate/empty-expect',
                component: ComponentCreator('/docs/detectors/substrate/empty-expect', 'f36'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/substrate/equal-addresses',
                component: ComponentCreator('/docs/detectors/substrate/equal-addresses', '3ca'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/substrate/invalid-extrinsic-weight',
                component: ComponentCreator('/docs/detectors/substrate/invalid-extrinsic-weight', 'a48'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/substrate/known-vulnerabilities',
                component: ComponentCreator('/docs/detectors/substrate/known-vulnerabilities', 'e7b'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/detectors/substrate/unsigned-extrinsic',
                component: ComponentCreator('/docs/detectors/substrate/unsigned-extrinsic', 'd95'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/features/command-line-interface',
                component: ComponentCreator('/docs/features/command-line-interface', 'fc4'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/features/profiles',
                component: ComponentCreator('/docs/features/profiles', '3bb'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/features/scout-github-action',
                component: ComponentCreator('/docs/features/scout-github-action', 'd34'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/features/toggle-detections',
                component: ComponentCreator('/docs/features/toggle-detections', 'fd1'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/features/vs-code-extension',
                component: ComponentCreator('/docs/features/vs-code-extension', '977'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/intro',
                component: ComponentCreator('/docs/intro', '61d'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/learning/learning-to-scout-soroban',
                component: ComponentCreator('/docs/learning/learning-to-scout-soroban', '964'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/learning/scout-soroban-examples',
                component: ComponentCreator('/docs/learning/scout-soroban-examples', 'bfc'),
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
    path: '/',
    component: ComponentCreator('/', '2e1'),
    exact: true
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];
