# Scout: Security Analysis Tool

[![Crates.io](https://img.shields.io/crates/v/cargo-scout-audit?label=crates.io)](https://crates.io/crates/cargo-scout-audit)
[![Docker](https://img.shields.io/docker/v/coinfabrik/scout?label=docker&sort=semver)](https://hub.docker.com/r/coinfabrik/scout)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](https://github.com/CoinFabrik/scout-audit/blob/main/LICENSE.txt)

<p align="center">
  <img src="https://raw.githubusercontent.com/CoinFabrik/scout-audit/c1eb3073f85b051dc9ce2fa0ab1ebab4bde0914e/assets/scout.png" alt="Scout in a dark forest" width="300" />
</p>

Scout is an extensible, open-source static analysis tool that helps developers and auditors find common security issues and deviations from best practices in **[Soroban](https://developers.stellar.org/docs/build/smart-contracts/overview)**, [ink!](https://use.ink/), and [Substrate pallets](https://docs.polkadot.com/develop/parachains/customize-parachain/).

Scout is available as a command-line tool, a VS Code extension, a GitHub Action, and a Docker image.

## What's new in 0.3.17

Scout 0.3.17 supports analysis of Soroban SDK 28 contracts. Scout automatically selects the `wasm32v1-none` target and supplies the SDK 28 spec-shaking-v2 build-system marker; projects can continue to use the normal `cargo scout-audit` command without extra target flags. Scout's default detector toolchain is `nightly-2025-09-18`.

This release includes three new Soroban detectors:

| Detector | Category | Severity | What it detects |
| --- | --- | --- | --- |
| [`delegated-spending-from-auth`](https://github.com/CoinFabrik/scout-audit/tree/089f4a511e35188aff4b85d1ea018beab85d9f98/nightly/2025-09-18/detectors/soroban/delegated-spending-from-auth) | Authorization | Minor | Delegated token transfers or burns that require both spender and owner authorization, defeating the allowance model. |
| [`init-instead-of-constructor`](https://github.com/CoinFabrik/scout-audit/tree/089f4a511e35188aff4b85d1ea018beab85d9f98/nightly/2025-09-18/detectors/soroban/init-instead-of-constructor) | Best practices | Medium | `init` or `initialize` entrypoints that should use Soroban's `__constructor` pattern to avoid initialization races and double initialization. |
| [`infinite-recursion-over-storage`](https://github.com/CoinFabrik/scout-audit/tree/089f4a511e35188aff4b85d1ea018beab85d9f98/nightly/2025-09-18/detectors/soroban/infinite-recursion-over-storage) | Denial of service | Medium | Direct or mutual recursion reachable from contract entrypoints that can exhaust gas or stack space. |

The release also makes Rust toolchain selection more deterministic, reports Dylint setup failures directly, supports targeted detector runs in CI, rejects empty detector test suites, and adds an option to suppress Scout's network requests. The matching [`coinfabrik/scout:0.3.17`](https://hub.docker.com/r/coinfabrik/scout/tags?name=0.3.17) image contains the pinned toolchain targets and detector sources.

## Quick start

Install [Rust and Cargo with `rustup`](https://www.rust-lang.org/tools/install). Scout relies on rustup-managed compiler components; package-manager-only Rust installations can conflict with the detector toolchain.

Install Scout from crates.io:

```bash
cargo install cargo-scout-audit --locked
```

Then run Scout from the directory that contains your project's `Cargo.toml`:

```bash
cargo scout-audit
```

Scout supports [Cargo workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) and analyzes the workspace's default members. The project must compile before Scout can analyze it.

See the [getting-started guide](https://coinfabrik.github.io/scout-audit/docs/intro) for more installation and usage details.

### Docker

The published image currently targets `linux/amd64`. From a project's root directory, run:

```bash
docker run --rm --platform linux/amd64 \
  -e INPUT_TARGET=/scoutme \
  -e INPUT_SCOUT_ARGS="--output-format html" \
  -v "$PWD:/scoutme" \
  coinfabrik/scout:0.3.17
```

Pin a numbered tag in automation. Use `coinfabrik/scout:latest` only when you explicitly want the newest release.

### Network controls

Set `SCOUT_OFFLINE` to `1`, `true`, or `yes` to disable telemetry and the release-update check:

```bash
SCOUT_OFFLINE=1 cargo scout-audit
```

This variable suppresses Scout's own telemetry and update requests; it does not by itself prevent Cargo or detector resolution from accessing the network. A network-isolated run also needs cached Rust/Cargo dependencies and local Scout sources:

```bash
SCOUT_OFFLINE=1 cargo scout-audit \
  --local-detectors /path/to/scout-audit/nightly \
  --scout-source /path/to/scout-audit
```

## Output formats

Scout prints its findings to the console by default. To generate report files, select one format or a comma-separated set with `--output-format`:

```bash
cargo scout-audit --output-format html
cargo scout-audit --output-format html,sarif
```

Supported values are `html`, `json`, `raw-json`, `raw-single-json`, `unfiltered-json`, `md`, `md-gh`, `sarif`, and `pdf`.

**Example HTML report**

<img src="https://raw.githubusercontent.com/CoinFabrik/scout-audit/766af703364d0a27cf69b5040a20f6c0b01b68f5/img/html.png" alt="Scout HTML report showing categorized findings" width="900" />

## Detectors

Scout runs shared Rust checks together with detectors tailored to **Soroban**, ink!, and Substrate pallets. The [detector catalog](https://coinfabrik.github.io/scout-audit/docs/detectors/detectors-intro) describes each issue, its severity, vulnerable examples, remediation, and detection strategy.

To inspect the detectors available for the current project, run:

```bash
cargo scout-audit --list-detectors
```

## Integrations

### VS Code extension

The [Scout VS Code extension](https://marketplace.visualstudio.com/items?itemName=CoinFabrik.scout-audit) can run Scout when a file is saved and show findings in the editor. Installing [Error Lens](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens) makes inline diagnostics more prominent.

<img src="https://raw.githubusercontent.com/CoinFabrik/scout-audit/089f4a511e35188aff4b85d1ea018beab85d9f98/img/vscode-extension.png" alt="Scout findings displayed in VS Code" width="900" />

### GitHub Action

The [Scout GitHub Action](https://github.com/marketplace/actions/run-scout-action) runs the analysis in CI and can publish findings on pull requests.

<img src="https://raw.githubusercontent.com/CoinFabrik/scout-audit/766af703364d0a27cf69b5040a20f6c0b01b68f5/img/github-action-output.jpg" alt="Scout GitHub Action findings in a pull-request comment" width="520" />

## Development and tests

The repository pins detector builds to `nightly-2025-09-18`. Install the local CLI from the repository root before running the detector test scripts:

```bash
cargo install \
  --path apps/cargo-scout-audit/crates/cargo-scout-audit \
  --locked
```

Validate the detector layout and metadata:

```bash
python3 scripts/validate-detectors.py
```

Run a single detector suite using the same `blockchain/detector` identifier accepted by the CI workflow:

```bash
python3 scripts/run-tests.py \
  --detector=soroban/infinite-recursion-over-storage
```

Run every detector suite, or the complete formatting, linting, and test pipeline:

```bash
make test
make ci
```

Vulnerable and remediated fixtures live under [`test-cases/<blockchain>/<detector>/`](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases).

## Acknowledgements

Scout is an open-source vulnerability analyzer developed by [CoinFabrik's](https://www.coinfabrik.com/) Research and Development team.

We received support from the **[Stellar Community Fund](https://communityfund.stellar.org)**, the [Web3 Foundation Grants Program](https://github.com/w3f/Grants-Program/tree/master), the [Aleph Zero Ecosystem Funding Program](https://alephzero.org/blog/introducing-ecosystem-funding-program), and [Polkadot Assurance Legion](https://polkadotassurance.com/).

| Grant program | Description |
| --- | --- |
| <img src="https://raw.githubusercontent.com/CoinFabrik/scout-audit/089f4a511e35188aff4b85d1ea018beab85d9f98/img/stellar.png" alt="Stellar Community Fund" height="80" /> | We added **Soroban support**, multiple report formats, precision and recall improvements, and a GitHub Action for pull requests. |
| <img src="https://raw.githubusercontent.com/CoinFabrik/scout-audit/c1eb3073f85b051dc9ce2fa0ab1ebab4bde0914e/assets/web3-foundation.png" alt="Web3 Foundation" height="80" /> | **Proof of concept:** We collaborated with the [Laboratory on Foundations and Tools for Software Engineering (LaFHIS)](https://lafhis.dc.uba.ar/) at the [University of Buenos Aires](https://www.uba.ar/) to establish analysis techniques and tools for Scout and create an initial list of vulnerability classes and examples. [View grant](https://github.com/CoinFabrik/web3-grant) \| [Application](https://github.com/w3f/Grants-Program/blob/master/applications/ScoutCoinFabrik.md).<br /><br />**Prototype:** We built a working prototype with [Dylint](https://github.com/trailofbits/dylint), then expanded its vulnerability classes, detectors, and test cases. [View prototype source](https://github.com/CoinFabrik/scout-audit/tree/c1eb3073f85b051dc9ce2fa0ab1ebab4bde0914e) \| [Application](https://github.com/w3f/Grants-Program/blob/master/applications/ScoutCoinFabrik_2.md). |
| <img src="https://raw.githubusercontent.com/CoinFabrik/scout-audit/c1eb3073f85b051dc9ce2fa0ab1ebab4bde0914e/assets/aleph-zero.png" alt="Aleph Zero" height="80" /> | We improved detector coverage and precision through manual analysis of projects in the Aleph Zero ecosystem, testing on leading projects, and detection-quality refinements. |
| <img src="https://raw.githubusercontent.com/CoinFabrik/scout-audit/089f4a511e35188aff4b85d1ea018beab85d9f98/img/PAL_logo.svg" alt="Polkadot Assurance Legion" height="80" /> | We added support for Substrate pallets across the CLI, VS Code extension, and GitHub Action. |

## About CoinFabrik

[CoinFabrik](https://www.coinfabrik.com/) is a Web3 research and development company with a strong cybersecurity background. Since 2014, its teams have worked across EVM, Solana, Algorand, Stellar, and Polkadot ecosystems and conduct security audits for multiple smart-contract and blockchain platforms.

The team has an academic background in computer science and mathematics, including publications, productized patents, conference presentations, and an ongoing collaboration with the University of Buenos Aires on knowledge transfer and open-source projects.

## License

Scout is distributed under the [MIT License](https://github.com/CoinFabrik/scout-audit/blob/main/LICENSE.txt).
