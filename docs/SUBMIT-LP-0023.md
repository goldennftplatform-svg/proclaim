# Submitting the LP-0023 solution PR — step by step

The solution body is drafted in [`docs/SOLUTION-LP-0023.md`](SOLUTION-LP-0023.md) in the proclaim repo. This guide records the exact steps to open it.

## State right now

- `proclaim` is pushed: <https://github.com/goldennftplatform-svg/proclaim>, remote HEAD `600bddc8c4f24a2b86fec0bc0b213b607021ac8e`.
- No solution PR for LP-0023 exists in `logos-co/lambda-prize` (checked 2026-11-19; only #187 for LP-0018, and spec/policy PRs #188/#189).
- The drafted PR is an **early draft** in the style of solution PR #187: it establishes priority while honestly marking unmet criteria.

## Before you open the PR — decide consciously

The rules give each builder **max 3 submissions per prize** and **one review per week**. A PR's priority is set by its **opening timestamp**, so opening early protects the "first solution wins" race. But a pass/fail review of an incomplete draft may consume one of the three allowances. PR #187 (LP-0018) took the early-draft route, so it is a recognised pattern — still, make this call deliberately.

Also check these two open PRs before submitting, because they could change the rules or the spec:

- **#188** "Update for lez 0.4, and updated idl" — proposes changing the LP-0023 spec for LEZ 0.4.
- **#189** "Release integrity and security requirements for submissions" — draft policy adding signed releases, a release-integrity manifest and security tiers. It explicitly does not touch existing prize files, but if it merges first, add a `## Release Integrity` section with commit SHA, catalog URL, package/version, release URL, `rootHash`, SHA-256, variants and signer DID.

## Steps

1. Fill in **`Submitted by:`** at the top of `docs/SOLUTION-LP-0023.md` with your real name or team name. The implementation commit `600bddc…` and all links are already correct and stable.

2. Fork <https://github.com/logos-co/lambda-prize> to your GitHub account (web UI: **Fork**). GitHub allows one fork per account, so if you already have a fork you must use it.

3. Clone your fork and create a branch:

   ```bash
   git clone https://github.com/<your-user>/lambda-prize.git
   cd lambda-prize
   git checkout -b solution/lp-0023-proclaim
   ```

4. Copy the drafted body into the required path (the file name must match the prize ID):

   ```bash
   cp <proclaim>/docs/SOLUTION-LP-0023.md solutions/LP-0023.md
   ```

5. Commit and push to your fork:

   ```bash
   git add solutions/LP-0023.md
   git commit -m "Solution: LP-0023 — proclaim program registry"
   git push -u origin solution/lp-0023-proclaim
   ```

6. Open the PR against `logos-co/lambda-prize:master` with this exact title:

   ```
   Solution: LP-0023 — proclaim: LEZ Program Registry with reproducible source verification
   ```

   Paste `solutions/LP-0023.md` as the PR body (GitHub will do this automatically if the file is the only change).

7. After the PR is opened, post a short build-in-public note in Logos Discord `#builder-hub` linking the PR. This is expected by the program and helps recruit the independent adopters the Adoption criteria require.

## After submission

- Do not open a payment claim until the solution PR is **merged** into `solutions/`.
- Once merged, use the [λPrize payment issue template](https://github.com/logos-co/lambda-prize/issues/new?template=lambda-prize-claim.yml). It asks for full legal name, country of residence and an Ethereum wallet address; the address is public in the issue, so a single-use address is recommended.
- Keep working the [Pending work](SOLUTION-LP-0023.md#pending-work) list. Update the same solution PR as criteria are met (respecting the one-review-per-week guidance) rather than opening a second PR unless necessary.

## Credential hygiene

The PAT used to push proclaim was exposed in chat and local scripts and has been deleted from disk. **Revoke it in GitHub → Settings → Developer settings → Personal access tokens** and issue a new one if you need further pushes. The λPrize fork push in step 5 will need that fresh credential.
