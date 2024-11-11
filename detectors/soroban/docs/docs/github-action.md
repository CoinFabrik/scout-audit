---
sidebar_position: 7
---

# Scout GitHub Action

At CoinFabrik, we understand the importance of ensuring code quality and security in every step of the development process. That's why we've developed a GitHub action to integrate Scout into the CI/CD pipeline.

Scout is triggered upon every commit pushed to a pull request, automatically running the tool against the targeted smart contracts. This immediate feedback loop allows developers to quickly address any issues before merging the code into the main branch, reducing the risk of introducing bugs or vulnerabilities.

## Quick Start

To integrate Scout into your CI/CD pipeline, simply add the following `scout.yml` to the `.github/workflows` directory in your repo.

```yml
name: scout-workflow
on: [push]

jobs:
  scout-audit:
    runs-on: ubuntu-latest
    permissions:
      pull-requests: write
      contents: write
      repository-projects: write
    steps:
      - name: checkout
        uses: actions/checkout@v2

      - name: do scout
        uses: coinfabrik/scout-actions@v2.4
        with:
          target: # Path to the root of your smart contract (e.g. contracts/token/)
          markdown_output: "true"

      - uses: mshick/add-pr-comment@v2.8.2
        with:
          message-path:  ${{ github.workspace }}/report.md
```

## Considerations

1. Make sure that your smart contract compiles correctly. Scout will not run if any compilation errors exist.
2. Check that `target` in `scout.yml` is set to the root of the smart contract (where the `Cargo.toml` file is).
3. To properly see Scout's results, make sure that you have an open pull request to which you are committing your changes, as Scout's results will be shown as a comment in the PR.

## Output Example

Scout results are display as a comment in the pull request.

![Scout Action output example.](../static/img/github-action-output.jpg)
