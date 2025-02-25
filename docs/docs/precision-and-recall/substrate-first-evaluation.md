# Scout on Substrate: First precision report

December, 2024.

## 1. Introduction

As part of the second milestone of our initiative to develop Scout static analyzer for Substrate, we aimed to assess Scout’s precision and recall across a diverse set of repositories. This analysis involved running Scout over multiple projects, including pallets and runtimes, and comparing its findings against audited and unaudited code segments to identify true positives, false positives, and compilation issues affecting the tool’s reliability.

To keep track of these analyses and their details, we created a dedicated [Excel spreadsheet](https://docs.google.com/spreadsheets/d/1a7yfwU206FeSToDt74d-l6gSzhO9IYih6LK6e-07Ajo) containing the auditor’s notes, true positive/false positive classifications, and additional context on each finding. Request access if you are interested in following the analysis more closely. The selected issues from audited projects considered for this analysis were filtered from the [Scout Substrate Dataset](https://github.com/CoinFabrik/scout-substrate-dataset), ensuring that the exact location of each issue in the code was informed in the audit report.

The version of Scout considered in this analysis corresponds to commit [54c2328caa0b65b30f31fc2e244ac2419a52f98f](https://github.com/CoinFabrik/scout-audit/tree/54c2328caa0b65b30f31fc2e244ac2419a52f98f).

## 2. Compilation Errors

During the evaluation process, Scout encountered several compilation issues when analyzing certain projects. Some minor issues were addressed by simply running `cargo update` to update dependencies. However, more complex errors were observed due to environmental incompatibilities, missing WASM targets, and misreporting statuses when the tool encountered compilation failures.

### 2.1. Common Types of Compilation Errors

| Type of Error                               | Description                                                                         | Example/Note                                        | Resolution Steps                                                                                                                                                                       |
| ------------------------------------------- | ----------------------------------------------------------------------------------- | --------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Dependency Conflicts / Missing Dependencies | Crates failing to compile due to out-of-date or conflicting dependencies.           | `proc-macro2` compilation error in Manta project    | Update dependencies (e.g., `cargo update`) or adjust Cargo.toml                                                                                                                        |
| WASM-Related Issues / Missing WASM STD      | Certain projects requiring WASM targets not properly configured in the environment. | Manta Network’s `wasm-test` environment issue       | Install WASM targets, ensure proper `rustup` setup, update env.                                                                                                                        |
| Scout Misreporting Errors as “Analyzed”     | Scout not accurately reflecting compilation failures in the summary report.         | Manta Network crate showing "Analyzed" despite fail | Addressed in [GitHub Issue #202](https://github.com/CoinFabrik/scout-audit/issues/202). Fixed status reporting in [GitHub PR #208](https://github.com/CoinFabrik/scout-audit/pull/208) |
| Unresolved Rust Environment Configurations  | Complex runtime environment or large monorepos causing partial build failures.      | Parallel, Astar, ReefChain builds failing           | Adjust tool environment, ensure all required toolchains present                                                                                                                        |

### 2.2. Steps Taken to Address Issues

Initial attempts to resolve dependency-related errors involved running `cargo update` and verifying the presence of required toolchains (e.g., installing `wasm32-unknown-unknown` targets). For more complex scenarios, like those found in the Manta Network repository, a dedicated GitHub issue was opened and the issue was resolved (See [GitHub Issue #202](https://github.com/CoinFabrik/scout-audit/issues/202) and [PR #208](https://github.com/CoinFabrik/scout-audit/pull/208)). This issue focused on ensuring Scout accurately reports the "Compilation errors" status when a crate fails to compile, rather than incorrectly labeling the crate as "Analyzed."

In addition to resolving reporting inaccuracies, the development team examined environmental and configuration issues. They worked on updating Scout’s handling of non-standard build environments and improving the logic that determines when a project is considered successfully analyzed versus when it is blocked by compilation hurdles.

These actions have enhanced Scout’s reliability and usability, ensuring that both minor and more systemic compilation issues are clearly surfaced and properly addressed in future analyses.

## 3. Precision and Recall by Detector

### 3.1. On Findings from Audited Projects

Of the total **N = 47** issues analyzed from audited projects extracted from the Scout Substrate Dataset, **M = 35** corresponded to code segments that could not be fully analyzed due to compilation errors. As mentioned in section `2. Compilation Errors` above, these errors were reviewed and are being followed (See [GitHub Issue #202](https://github.com/CoinFabrik/scout-audit/issues/202) and [PR #208](https://github.com/CoinFabrik/scout-audit/pull/208)) to improve Scout. This left **N - M = 12** issues associated with code segments where Scout could run without errors. All 12 issues were associated with the `integer_overflow_or_underflow` detector.

For these 12 issues, our auditor first reviewed each audit finding from which the analyzed code was extracted, determining whether Scout should detect the issue at the indicated line of code or not.

Our auditor’s analysis revealed that 6 of these issues were true positives—Scout accurately detected a genuine issue—while the other 6 were true negatives, as the flagged issues, despite being highlighted in audits, did not represent a detection point for the actual vulnerability and where correctly not detected by Scout.

### 3.2. On Scout Findings from Unaudited Projects

In addition to analyzing issues extracted from audited projects, our auditor also reviewed Scout’s detections on newer commits from the same set of projects where the tool now compiled and ran successfully, but whose code segments were not previously audited. This resulted in the analysis of an additional set of **35** detections.

For each detection, the auditor assessed whether it represented a true vulnerability (true positive) or if Scout incorrectly flagged a safe code pattern (false positive). In cases where the code was deemed non-vulnerable, and Scout should not have reported it, these detections were classified as false positives. No false negatives were considered here, as all findings listed are those that Scout flagged.

The following table summarizes the results by detector:

| Detector                      | True Positives | False Positives | % False Positives |
| ----------------------------- | -------------- | --------------- | ----------------- |
| known_vulnerabilities         | 1              | 0               | 0%                |
| avoid_panic_error             | 2              | 0               | 0%                |
| avoid-dispatcherror-other     | 3              | 0               | 0%                |
| assert_violation              | 7              | 0               | 0%                |
| unsafe_expect                 | 0              | 17              | 100%              |
| integer_overflow_or_underflow | 0              | 5               | 100%              |

As shown above, while several detectors produced valid results with no false positives, others—particularly `unsafe_expect` and `integer_overflow_or_underflow`—yielded a high number of false positives. This insight helps guide future improvements to the detectors, aiming to reduce incorrect alerts and improve the overall reliability of Scout.

## 4. Conclusion

This precision and recall exercise is a critical component in refining Scout’s capabilities as we progress through Milestone 2 of our project. By running the tool against a variety of audited and unaudited projects, we have identified both the technical barriers—such as compilation errors and environmental setup issues—and the areas where our detectors need further refinement to minimize false positives and misclassifications.

The results obtained here will inform the development of new detectors, improvements to existing ones, and enhancements to the supporting frameworks for analysis. Addressing these early findings will significantly improve Scout’s usability, ensuring that developers and auditors can rely on its insights. Ultimately, by identifying precision and environment-related issues now, we are setting a strong foundation for the subsequent milestones, which will deliver a more robust and accurate tool.
